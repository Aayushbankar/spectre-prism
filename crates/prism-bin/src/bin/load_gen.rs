use std::net::UdpSocket;
use std::time::Instant;
use std::sync::Arc;
use std::thread;

fn main() {
    let target = "127.0.0.1:5514";
    let iterations = 250_000; // Total logs to send per thread
    let threads = 4; // Max out 4 cores to blast data
    
    // Real Fortinet log sample
    let payload = "<134>date=2024-01-15 time=08:23:41 devname=\"FGT-DC-01\" devid=\"FG3H0E5018902345\" logid=\"0000000013\" type=\"traffic\" subtype=\"forward\" level=\"notice\" vd=\"root\" eventtime=1705312221 srcip=10.10.20.45 srcport=52341 srcintf=\"port5\" srcintfpolicy=\"lan\" dstip=203.0.113.25 dstport=443 dstintf=\"port1\" dstintfpolicy=\"wan\" poluuid=\"a1b2c3d4-e5f6-7890-abcd-ef1234567890\" sessionid=847293651 proto=6 action=\"accept\" policyid=15 policyname=\"LAN-to-Internet\" user=\"jsmith\" group=\"domain-users\" appcat=\"Web.Client\" crscore=5 craction=0 sentbyte=15234 rcvdbyte=892451 sentpkt=124 rcvdpkt=612 duration=45\n";

    println!("Starting real load generation against {}", target);
    println!("Payload size: {} bytes", payload.len());
    println!("Threads: {}, Messages per thread: {}", threads, iterations);
    println!("Total Messages: {}", threads * iterations);

    let start_time = Instant::now();
    let mut handles = vec![];

    for i in 0..threads {
        let payload_clone = payload.to_string();
        let target_clone = target.to_string();
        
        let handle = thread::spawn(move || {
            let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind UDP socket");
            let bytes = payload_clone.as_bytes();
            let mut sent = 0;
            
            for _ in 0..iterations {
                if socket.send_to(bytes, &target_clone).is_ok() {
                    sent += 1;
                }
            }
            println!("Thread {} finished: sent {}", i, sent);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let elapsed = start_time.elapsed().as_secs_f64();
    let total_sent = threads * iterations;
    let eps = (total_sent as f64 / elapsed) as u64;

    println!("=========================================");
    println!("⏱️ Load Generation Complete");
    println!("Time Elapsed : {:.2} seconds", elapsed);
    println!("Total Sent   : {}", total_sent);
    println!("Target EPS   : {} EPS", eps);
    println!("=========================================");
}
