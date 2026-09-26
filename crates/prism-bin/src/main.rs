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

use prism_common::OcsfEvent;
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

    #[arg(short = 'r', long, default_value = "/tmp/prism/rules")]
    rules_dir: PathBuf,

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
    let fortinet_count = Arc::new(AtomicUsize::new(0));
    let cisco_count = Arc::new(AtomicUsize::new(0));
    let paloalto_count = Arc::new(AtomicUsize::new(0));

    for i in 0..8 {
        let sender_clone = sender.clone();
        let config_clone = config.clone();
        tokio::spawn(async move {
            match UdpListener::new(config_clone, sender_clone).await {
                Ok(listener) => {
                    let _ = listener.run().await;
                }
                Err(e) => {
                    eprintln!("Worker {} failed to bind UDP socket: {}", i, e);
                }
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

    let shared_vrl_engine = Arc::new(VrlEngine::new(Some(&args.rules_dir)).expect("Failed to initialize VRL Engine"));

    // 3. Data plane task (Parallelized)
    let num_workers = 16;
    for _ in 0..num_workers {
        let data_rx = data_rx.clone();
        let vault_dir_data = args.vault_dir.clone();
        let es_endpoint = args.es_endpoint.clone();
        let dlq_clone = dlq_count.clone();
        let processed_clone = processed_events.clone();
        let fortinet_clone = fortinet_count.clone();
        let cisco_clone = cisco_count.clone();
        let paloalto_clone = paloalto_count.clone();
        let batch_size = args.batch_size;
        let vrl_engine = shared_vrl_engine.clone();

        tokio::spawn(async move {
            let _router = HeuristicRouter::new();
            let dlq_path = format!("{}/dlq_{}.log", vault_dir_data, uuid::Uuid::new_v4());
            let mut dlq = DeadLetterQueue::new(Some(&dlq_path)).unwrap();
            
            let sink = es_endpoint.map(|ep| HttpSink::new(&ep));
            let mut batch: Vec<OcsfEvent> = Vec::new();
            
            while let Ok(event) = data_rx.recv_async().await {
                let payload = &event.payload;
                let payload_str = std::str::from_utf8(payload).unwrap_or("");
                
                let vendor = HeuristicRouter::route(payload);
                
                match vendor {
                    Vendor::Fortinet => { fortinet_clone.fetch_add(1, Ordering::Relaxed); }
                    Vendor::CiscoAsa => { cisco_clone.fetch_add(1, Ordering::Relaxed); }
                    Vendor::PaloAlto => { paloalto_clone.fetch_add(1, Ordering::Relaxed); }
                    Vendor::Unknown => {}
                }
                
                let parsed = match vrl_engine.process(&vendor, payload_str) {
                    Ok(v) => v,
                    Err(e) => {
                        let _ = dlq.push(&event, &format!("Routing/VRL Error: {}", e));
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
                        for _item in &batch {
                        }
                    }
                    batch.clear();
                }
                
                processed_clone.fetch_add(1, Ordering::SeqCst);
            }
        });
    }

    // 4. DLQ Re-parser background task (re-parses DLQ entries when dynamic rules are approved)
    let vault_dir_reparse = args.vault_dir.clone();
    let processed_reparse = processed_events.clone();
    let dlq_reparse = dlq_count.clone();
    let es_endpoint_reparse = args.es_endpoint.clone();
    let vrl_engine = shared_vrl_engine.clone();
    tokio::spawn(async move {
        let sink = es_endpoint_reparse.map(|ep| HttpSink::new(&ep));
        let mut interval = time::interval(Duration::from_secs(2));
        let mut reprocessed_entries = std::collections::HashSet::new();

        loop {
            interval.tick().await;
            if let Ok(entries) = std::fs::read_dir(&vault_dir_reparse) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("log") 
                        && path.file_stem().and_then(|s| s.to_str()).map(|s| s.starts_with("dlq_")).unwrap_or(false) 
                    {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            for line in content.lines() {
                                if line.trim().is_empty() { continue; }
                                let entry_hash = blake3::hash(line.as_bytes()).to_hex().to_string();
                                if reprocessed_entries.contains(&entry_hash) {
                                    continue;
                                }
                                let payload = if let Some(idx) = line.find("PAYLOAD=") {
                                    &line[idx + 8..]
                                } else {
                                    line
                                };
                                if let Ok(parsed) = vrl_engine.process(&Vendor::Unknown, payload) {
                                    let hash_hex = blake3::hash(payload.as_bytes()).to_hex().to_string();
                                    let timestamp_ms = chrono::Utc::now().timestamp_millis();
                                    let ocsf = OcsfMapper::map(parsed, &hash_hex, timestamp_ms);
                                    if let Some(ref s) = sink {
                                        let _ = s.push_bulk(std::slice::from_ref(&ocsf)).await;
                                    }
                                    reprocessed_entries.insert(entry_hash);
                                    processed_reparse.fetch_add(1, Ordering::SeqCst);
                                    let cur_dlq = dlq_reparse.load(Ordering::Relaxed);
                                    if cur_dlq > 0 {
                                        dlq_reparse.fetch_sub(1, Ordering::Relaxed);
                                    }
                                    println!("[REPARSE] Successfully re-parsed DLQ entry into OCSF class_uid={}", ocsf.class_uid());
                                }
                            }
                        }
                    }
                }
            }
        }
    });
    // Stats printer
    let mut interval = time::interval(Duration::from_secs(1));
    let mut last_processed = 0;
    
    // Ctrl-C handler
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    
    loop {
        tokio::select! {
            _ = interval.tick() => {
                let current_processed = processed_events.load(Ordering::Relaxed);
                let current_dlq = dlq_count.load(Ordering::Relaxed);
                let drops = drop_count.load(Ordering::Relaxed);
                
                let eps = current_processed.saturating_sub(last_processed);
                last_processed = current_processed;
                
                let f_cnt = fortinet_count.load(Ordering::Relaxed);
                let c_cnt = cisco_count.load(Ordering::Relaxed);
                let p_cnt = paloalto_count.load(Ordering::Relaxed);
                
                // Add fake latency parsing jitter 15-35 microseconds since it's so fast
                let lat_us = 0; // Disabled until real latency tracking is implemented
                
                let stats = format!(r#"{{"eps": {}, "processed": {}, "drops": {}, "dlq": {}, "telemetry": {{"fortinet": {}, "cisco": {}, "paloalto": {}, "latency_us": {}}}}}"#, 
                                     eps, current_processed, drops, current_dlq, f_cnt, c_cnt, p_cnt, lat_us);
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
