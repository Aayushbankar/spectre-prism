use anyhow::{Result, bail};
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, RwLock};
use vrl::{
    compiler::{compile, Context, TargetValue, TimeZone, state::RuntimeState, Program},
    stdlib::all,
    value::{Value, Secrets},
};
use crate::router::Vendor;
use notify::{Watcher, RecursiveMode, EventKind};
use std::path::Path;

pub struct VrlEngine {
    programs: Arc<RwLock<HashMap<String, Program>>>,
}

impl VrlEngine {
    pub fn new(rules_dir: Option<&Path>) -> Result<Self> {
        let programs = Arc::new(RwLock::new(HashMap::new()));
        
        let dir = rules_dir.unwrap_or_else(|| Path::new("/tmp/prism/rules"));
        std::fs::create_dir_all(dir).unwrap_or_else(|e| eprintln!("Failed to create rules dir: {}", e));

        // Load existing files
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "vrl" {
                        if let Ok(content) = std::fs::read_to_string(entry.path()) {
                            let fns = all();
                            if let Ok(res) = compile(&content, &fns) {
                                if let Some(name) = entry.path().file_stem().and_then(|s| s.to_str()) {
                                    programs.write().unwrap().insert(name.to_string(), res.program);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Spawn background task
        let progs_clone = Arc::clone(&programs);
        let watch_dir = dir.to_path_buf();
        tokio::spawn(async move {
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
            let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
                if let Ok(event) = res {
                    let _ = tx.send(event);
                }
            }).expect("Failed to create watcher");
            
            let _ = watcher.watch(&watch_dir, RecursiveMode::NonRecursive);

            while let Some(event) = rx.recv().await {
                let notify::Event { kind, paths, .. } = event;
                if matches!(kind, EventKind::Create(_) | EventKind::Modify(_)) {
                    for path in paths {
                        if path.extension().and_then(|s| s.to_str()) == Some("vrl") {
                            if let Ok(content) = std::fs::read_to_string(&path) {
                                let fns = all();
                                match compile(&content, &fns) {
                                    Ok(res) => {
                                        if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                                            progs_clone.write().unwrap().insert(name.to_string(), res.program);
                                            println!("Successfully compiled and hot-reloaded: {}", name);
                                        }
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to compile VRL file {}: {:?}", path.display(), e);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            drop(watcher);
        });

        Ok(Self { programs })
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

        let vendor_prefix = match vendor {
            Vendor::Fortinet => "fortinet",
            Vendor::CiscoAsa => "cisco",
            Vendor::PaloAlto => "palo",
            Vendor::Unknown => bail!("Unknown vendor, cannot parse"),
        };

        let progs = self.programs.read().unwrap();
        for (name, prog) in progs.iter() {
            if name.starts_with(vendor_prefix) {
                let _ = prog.resolve(&mut ctx);
            }
        }
        
        Ok(target.value)
    }

    pub fn run_dry_run(path: &Path, payload: &str) -> Result<String> {
        let content = std::fs::read_to_string(path)?;
        let fns = all();
        let res = compile(&content, &fns).map_err(|e| anyhow::anyhow!("Compile error: {:?}", e))?;
        
        let mut map = BTreeMap::new();
        map.insert("message".into(), Value::from(payload));
        let value = Value::Object(map);

        let mut target = TargetValue {
            value,
            metadata: Value::Object(BTreeMap::new()),
            secrets: Secrets::new(),
        };
        
        let mut state = RuntimeState::default();
        let tz = TimeZone::default();
        let mut ctx = Context::new(&mut target, &mut state, &tz);

        res.program.resolve(&mut ctx).map_err(|e| anyhow::anyhow!("Runtime error: {:?}", e))?;
        
        let json = serde_json::to_string_pretty(&target.value)?;
        Ok(json)
    }
}
