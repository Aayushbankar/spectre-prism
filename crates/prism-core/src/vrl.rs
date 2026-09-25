use anyhow::{Result, bail};
use std::collections::BTreeMap;
use vrl::{
    compiler::{compile, Context, TargetValue, TimeZone, state::RuntimeState, Program},
    stdlib::all,
    value::{Value, Secrets},
};
use crate::router::Vendor;

pub struct VrlEngine {
    fortinet_programs: Vec<Program>,
    cisco_programs: Vec<Program>,
    palo_program: Program,
}

impl VrlEngine {
    pub fn new() -> Result<Self> {
        let fortinet_scripts = vec![
            r#".srcip = parse_regex!(string!(.message), r'srcip=(?P<srcip>\d+\.\d+\.\d+\.\d+)').srcip"#,
            r#".dstip = parse_regex!(string!(.message), r'dstip=(?P<dstip>\d+\.\d+\.\d+\.\d+)').dstip"#,
            r#".srcport = parse_regex!(string!(.message), r'srcport=(?P<srcport>\d+)').srcport"#,
            r#".dstport = parse_regex!(string!(.message), r'dstport=(?P<dstport>\d+)').dstport"#,
            r#".action = parse_regex!(string!(.message), r'action="(?P<action>[^"]+)"').action"#,
            r#".proto = parse_regex!(string!(.message), r'proto=(?P<proto>\d+)').proto"#,
            r#".sentbyte = parse_regex!(string!(.message), r'sentbyte=(?P<sentbyte>\d+)').sentbyte"#,
            r#".rcvdbyte = parse_regex!(string!(.message), r'rcvdbyte=(?P<rcvdbyte>\d+)').rcvdbyte"#,
            r#".level = parse_regex!(string!(.message), r'level="(?P<level>[^"]+)"').level"#,
        ];

        let cisco_scripts = vec![
            r#".srcip = parse_regex!(string!(.message), r'outside:(?P<srcip>\d+\.\d+\.\d+\.\d+)').srcip"#,
            r#".srcport = parse_regex!(string!(.message), r'outside:\d+\.\d+\.\d+\.\d+/(?P<srcport>\d+)').srcport"#,
            r#".dstip = parse_regex!(string!(.message), r'inside:(?P<dstip>\d+\.\d+\.\d+\.\d+)').dstip"#,
            r#".dstport = parse_regex!(string!(.message), r'inside:\d+\.\d+\.\d+\.\d+/(?P<dstport>\d+)').dstport"#,
            r#".msgid = parse_regex!(string!(.message), r'%ASA-\d-(?P<msgid>\d+)').msgid"#,
        ];

        let palo_script = r#"
            parts = split(string!(.message), ",")
            .srcip = parts[7]
            .dstip = parts[8]
            .srcport = parts[24]
            .dstport = parts[25]
            .action = parts[29]
        "#;

        let fns = all();
        
        let fortinet_programs: Result<Vec<Program>, _> = fortinet_scripts.into_iter()
            .map(|s| compile(s, &fns).map(|res| res.program))
            .collect();
        let fortinet_programs = fortinet_programs.map_err(|_| anyhow::anyhow!("Fortinet VRL compile error"))?;

        let cisco_programs: Result<Vec<Program>, _> = cisco_scripts.into_iter()
            .map(|s| compile(s, &fns).map(|res| res.program))
            .collect();
        let cisco_programs = cisco_programs.map_err(|_| anyhow::anyhow!("Cisco VRL compile error"))?;

        let palo_program = compile(palo_script, &fns).map_err(|_| anyhow::anyhow!("Palo VRL compile error"))?.program;
        
        Ok(Self {
            fortinet_programs,
            cisco_programs,
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

        match vendor {
            Vendor::Fortinet => {
                for prog in &self.fortinet_programs {
                    let _ = prog.resolve(&mut ctx);
                }
            }
            Vendor::CiscoAsa => {
                for prog in &self.cisco_programs {
                    let _ = prog.resolve(&mut ctx);
                }
            }
            Vendor::PaloAlto => {
                let _ = self.palo_program.resolve(&mut ctx);
            }
            Vendor::Unknown => bail!("Unknown vendor, cannot parse"),
        }
        
        Ok(target.value)
    }
}
