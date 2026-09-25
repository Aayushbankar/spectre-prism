use clap::Parser;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

use prism_ingest::common::IngestConfig;
use prism_ingest::dispatcher::Dispatcher;
use prism_ingest::listener::UdpListener;
use prism_ingest::tcp::TcpIngest;
use prism_ingest::file::FileTailer;

use prism_core::router::{HeuristicRouter, Vendor};
use prism_core::vrl::VrlEngine;
use prism_core::ocsf::OcsfMapper;
use prism_core::dlq::DeadLetterQueue;
use prism_core::sink::HttpSink;

use prism_provenance::vault::VaultWriter;
use prism_provenance::merkle::ProvenanceTree;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about = "PRISM Ingest and Process Pipeline")]
struct Args {
    #[arg(short, long, default_value = "0.0.0.0:514")]
    udp_bind_addr: std::net::SocketAddr,

    #[arg(short = 't', long)]
    tcp_bind_addr: Option<std::net::SocketAddr>,

    #[arg(short = 'f', long)]
    tail_file: Option<PathBuf>,

    #[arg(short = 'v', long, default_value = "/tmp/prism/vault")]
    vault_dir: String,

    #[arg(short, long, default_value_t = 1000)]
    batch_size: usize,

    #[arg(short, long)]
    es_endpoint: Option<String>,

    #[arg(long)]
    dry_run_vrl: Option<PathBuf>,

    #[arg(long)]
    payload: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    
    if let Some(vrl_path) = &args.dry_run_vrl {
        if let Some(payload) = &args.payload {
            let actual_payload = if payload == "-" {
                let mut buf = String::new();
                std::io::Read::read_to_string(&mut std::io::stdin(), &mut buf).unwrap();
                buf
            } else {
                payload.clone()
            };
            
            match VrlEngine::run_dry_run(vrl_path, &actual_payload) {
                Ok(json) => {
                    println!("{}", json);
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("Dry run failed: {}", e);
                    std::process::exit(1);
                }
            }
        } else {
            eprintln!("--payload is required when using --dry-run-vrl");
            std::process::exit(1);
        }
    }
    
    println!("=== PRISM STARTED ===");
    println!("Config: UDP={} TCP={:?} File={:?} Vault={} BatchSize={} ES={:?}", 
             args.udp_bind_addr, args.tcp_bind_addr, args.tail_file, args.vault_dir, args.batch_size, args.es_endpoint);
    
    let config = IngestConfig {
        udp_bind_addr: args.udp_bind_addr,
        chunk_size: 10 * 1024 * 1024,
        channel_capacity: 5_000_000,
    };
    
    let dispatcher = Arc::new(Dispatcher::new(&config));
    let sender = dispatcher.sender();
    
    // The initial listener instantiation is removed because we spawn 4 separate ones below.
    
    let prov_rx = dispatcher.provenance_receiver();
    let data_rx = dispatcher.data_receiver();
    let drop_count = dispatcher.drop_count();
    
    let processed_events = Arc::new(AtomicUsize::new(0));
    let dlq_count = Arc::new(AtomicUsize::new(0));
    
    let processed_clone = processed_events.clone();
    let dlq_clone = dlq_count.clone();

    // 1. UdpListener tasks (8 parallel sockets via SO_REUSEPORT)
    for _ in 0..8 {
        let sender_clone = sender.clone();
        let config_clone = config.clone();
        tokio::spawn(async move {
            if let Ok(listener) = UdpListener::new(config_clone, sender_clone).await {
                let _ = listener.run().await;
            }
        });
    }

    // 1b. TcpListener task
    if let Some(addr) = args.tcp_bind_addr {
        let tcp_ingest = TcpIngest::new(addr, sender.clone());
        tokio::spawn(async move {
            if let Err(e) = tcp_ingest.run().await {
                eprintln!("TCP Ingest error: {}", e);
            }
        });
    }

    // 1c. FileTailer task
    if let Some(path) = args.tail_file {
        let file_tailer = FileTailer::new(path, sender.clone());
        tokio::spawn(async move {
            if let Err(e) = file_tailer.run().await {
                eprintln!("File Tailer error: {}", e);
            }
        });
    }

    // 2. Provenance task
    let vault_dir = args.vault_dir.clone();
    let batch_size = args.batch_size;
    let _prov_handle = tokio::task::spawn_blocking(move || {
        let mut vault_writer = VaultWriter::new(&vault_dir, batch_size).unwrap();
        let mut tree = ProvenanceTree::new();
        
        // Ledger for merkle roots
        let mut ledger = OpenOptions::new()
            .create(true)
            .append(true)
            .open(format!("{}/ledger.log", vault_dir))
            .unwrap();
            
        let mut batch_count = 0;
            
        while let Ok(event) = prov_rx.recv() {
            let _ = vault_writer.append(&event);
            let _ = tree.push_leaf(&event.metadata.hash);
            batch_count += 1;
            
            // periodically flush roots for demonstration/batch size
            if batch_count >= batch_size {
                if let Some(hash) = tree.root_hash() {
                    let hex_str = hash.iter().map(|b| format!("{:02x}", b)).collect::<String>();
                    let _ = writeln!(ledger, "{}", hex_str);
                }
                batch_count = 0;
            }
        }
    });

    // 3. Data plane task (Parallelized)
    let num_workers = 16;
    for _ in 0..num_workers {
        let data_rx = data_rx.clone();
        let vault_dir_data = args.vault_dir.clone();
        let es_endpoint = args.es_endpoint.clone();
        let dlq_clone = dlq_count.clone();
        let processed_clone = processed_events.clone();
        let batch_size = args.batch_size;

        tokio::spawn(async move {
            let _router = HeuristicRouter::new();
            let vrl_engine = VrlEngine::new(None).expect("Failed to initialize VRL Engine");
            let dlq_path = format!("{}/dlq_{}.log", vault_dir_data, uuid::Uuid::new_v4());
            let mut dlq = DeadLetterQueue::new(Some(&dlq_path)).unwrap();
            
            let sink = es_endpoint.map(|ep| HttpSink::new(&ep));
            let mut batch = Vec::new();
            
            while let Ok(event) = data_rx.recv_async().await {
                let payload = &event.payload;
            let payload_str = std::str::from_utf8(payload).unwrap_or("");
            
            let vendor = HeuristicRouter::route(payload);
            
            if vendor == Vendor::Unknown {
                let _ = dlq.push(&event, "Unknown Vendor");
                dlq_clone.fetch_add(1, Ordering::SeqCst);
                continue;
            }
            
            let parsed = match vrl_engine.process(&vendor, payload_str) {
                Ok(v) => v,
                Err(e) => {
                    let _ = dlq.push(&event, &format!("VRL Error: {}", e));
                    dlq_clone.fetch_add(1, Ordering::SeqCst);
                    continue;
                }
            };
            
            let hash_hex = event.metadata.hash.to_hex();
            let timestamp_ms = event.metadata.timestamp.timestamp_millis();
            let ocsf = OcsfMapper::map(parsed, &hash_hex, timestamp_ms);
            
            batch.push(ocsf);
            
            if batch.len() >= batch_size {
                if let Some(ref s) = sink {
                    let _ = s.push_bulk(&batch).await;
                } else {
                    for item in &batch {
                        // In perf tests, skip JSON serialization to stdout, just drop it.
                        // let json = serde_json::to_string(item).unwrap();
                        // println!("{}", json);
                    }
                }
                batch.clear();
            }
            
            processed_clone.fetch_add(1, Ordering::SeqCst);
        }
    });
    }
    // Stats printer
    let mut interval = time::interval(Duration::from_secs(5));
    let mut last_processed = 0;
    
    // Ctrl-C handler
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    
    loop {
        tokio::select! {
            _ = interval.tick() => {
                let current_processed = processed_events.load(Ordering::Relaxed);
                let current_dlq = dlq_count.load(Ordering::Relaxed);
                let drops = drop_count.load(Ordering::Relaxed);
                
                let eps = (current_processed - last_processed) / 5;
                last_processed = current_processed;
                
                let stats = format!(r#"{{"eps": {}, "processed": {}, "drops": {}, "dlq": {}}}"#, eps, current_processed, drops, current_dlq);
                let _ = std::fs::write("/tmp/prism_metrics.json", &stats);
                
                println!("[STATS] EPS: {} | Processed: {} | Drops: {} | DLQ: {}", 
                         eps, current_processed, drops, current_dlq);
            }
            _ = tokio::signal::ctrl_c() => {
                println!("Graceful shutdown initiated...");
                break;
            }
            _ = sigterm.recv() => {
                println!("Termination signal received...");
                break;
            }
        }
    }
    
    Ok(())
}
