use memchr::memmem;

#[derive(Debug, PartialEq, Eq, Default)]
pub enum Vendor {
    Fortinet,
    CiscoAsa,
    PaloAlto,
    #[default]
    Unknown,
}

#[derive(Default)]
pub struct HeuristicRouter;

impl HeuristicRouter {
    pub fn new() -> Self {
        Self
    }

    /// Fast byte pattern heuristic matching
    pub fn route(payload: &[u8]) -> Vendor {
        if memmem::find(payload, b"Fortinet").is_some() 
            || memmem::find(payload, b"devname=").is_some() 
            || memmem::find(payload, b"logid=\"").is_some() {
            return Vendor::Fortinet;
        }
        
        if memmem::find(payload, b"%ASA-").is_some() {
            return Vendor::CiscoAsa;
        }
        
        if memmem::find(payload, b",THREAT").is_some() || memmem::find(payload, b",TRAFFI").is_some() {
            return Vendor::PaloAlto;
        }
        
        Vendor::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Note: requires nightly for #[bench] or we can just add a standard #[test] that runs a loop
    #[test]
    fn bench_router_heuristic() {
        let payload = b"date=2024-01-01 time=12:00:00 devname=\"FW01\" devid=\"FG100\" logid=\"0000000013\" type=\"traffic\" subtype=\"forward\" level=\"notice\" srcip=192.168.1.5 dstip=8.8.8.8 action=\"accept\"";
        let start = std::time::Instant::now();
        for _ in 0..1_000_000 {
            std::hint::black_box(HeuristicRouter::route(payload));
        }
        let duration = start.elapsed();
        println!("1 million routes took {:?}", duration);
        assert!(duration.as_secs() < 15, "Bench took too long: {:?}", duration);
    }
}
