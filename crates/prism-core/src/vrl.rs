use anyhow::{Result, bail};
use std::collections::BTreeMap;
use vrl::{
    compiler::{compile, Context, TargetValue, TimeZone, state::RuntimeState, Program},
    stdlib::all,
    value::{Value, Secrets},
};
use crate::router::Vendor;

pub struct VrlEngine {
    fortinet_program: Program,
    cisco_program: Program,
    palo_program: Program,
}

impl VrlEngine {
    pub fn new() -> Result<Self> {
        // Fallible operations in VRL use `!` but we want to ignore errors if it doesn't match perfectly,
        // or just let it fail. Actually parse_regex! doesn't return an error in VRL 0.35 if coalesced? 
        // Wait, the error coalescing `??` caused a compile error. We'll just use parse_regex!()
        // and if it aborts at runtime, we return the original value.
        let fortinet_script = r#"
            .ip = parse_regex!(string!(.message), r'srcip=(?P<ip>\d+\.\d+\.\d+\.\d+)').ip
        "#;
        let cisco_script = r#"
            .ip = parse_regex!(string!(.message), r'outside:(?P<ip>\d+\.\d+\.\d+\.\d+)').ip
        "#;
        let palo_script = r#"
            .ip = parse_regex!(string!(.message), r',(?P<ip>\d+\.\d+\.\d+\.\d+),').ip
        "#;

        let fns = all();
        let fortinet_program = compile(fortinet_script, &fns).map_err(|_| anyhow::anyhow!("Fortinet VRL compile error"))?.program;
        let cisco_program = compile(cisco_script, &fns).map_err(|_| anyhow::anyhow!("Cisco VRL compile error"))?.program;
        let palo_program = compile(palo_script, &fns).map_err(|_| anyhow::anyhow!("Palo VRL compile error"))?.program;
        
        Ok(Self {
            fortinet_program,
            cisco_program,
            palo_program,
        })
    }

    pub fn process(&self, vendor: &Vendor, raw_log: &str) -> Result<Value> {
        let mut map = BTreeMap::new();
        map.insert("message".into(), Value::from(raw_log));
        let value = Value::Object(map);

        let mut target = TargetValue {
            value,
            metadata: Value::Object(BTreeMap::new()),
            secrets: Secrets::new(),
        };
        
        let mut state = RuntimeState::default();
        let tz = TimeZone::default();
        let mut ctx = Context::new(&mut target, &mut state, &tz);

        let res = match vendor {
            Vendor::Fortinet => self.fortinet_program.resolve(&mut ctx),
            Vendor::CiscoAsa => self.cisco_program.resolve(&mut ctx),
            Vendor::PaloAlto => self.palo_program.resolve(&mut ctx),
            Vendor::Unknown => bail!("Unknown vendor, cannot parse"),
        };
        
        // If VRL fails at runtime, we just return the unparsed object.
        if let Err(e) = res {
            bail!("VRL runtime error: {:?}", e);
        }
        
        Ok(target.value)
    }
}
