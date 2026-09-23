#[derive(Debug, PartialEq, Eq)]
pub enum Vendor {
    Fortinet,
    CiscoAsa,
    PaloAlto,
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
        if payload.windows(8).any(|w| w == b"Fortinet" || w == b"devname=" || w == b"logid=\"0") {
            return Vendor::Fortinet;
        }
        
        if payload.windows(5).any(|w| w == b"%ASA-") {
            return Vendor::CiscoAsa;
        }
        
        // Palo Alto heuristic: usually CSV format with THREAT, TRAFFIC, etc.
        // Assuming typical PAN-OS CSV prefix or known marker
        if payload.windows(7).any(|w| w == b",THREAT" || w == b",TRAFFI") {
            return Vendor::PaloAlto;
        }
        
        Vendor::Unknown
    }
}
