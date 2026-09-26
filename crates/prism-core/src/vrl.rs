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

            let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
            loop {
                tokio::select! {
                    Some(event) = rx.recv() => {
                        let notify::Event { kind, paths, .. } = event;
                        if matches!(kind, EventKind::Create(_) | EventKind::Modify(_) | EventKind::Any) {
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
                    _ = interval.tick() => {
                        if let Ok(entries) = std::fs::read_dir(&watch_dir) {
                            for entry in entries.flatten() {
                                if entry.path().extension().and_then(|s| s.to_str()) == Some("vrl") {
                                    if let Some(name) = entry.path().file_stem().and_then(|s| s.to_str()) {
                                        let already_loaded = progs_clone.read().unwrap().contains_key(name);
                                        if !already_loaded {
                                            if let Ok(content) = std::fs::read_to_string(entry.path()) {
                                                let fns = all();
                                                if let Ok(res) = compile(&content, &fns) {
                                                    progs_clone.write().unwrap().insert(name.to_string(), res.program);
                                                    println!("Rescan successfully compiled and hot-reloaded: {}", name);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
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

        let progs = self.programs.read().unwrap();
        match vendor {
            Vendor::Fortinet => {
                for (name, prog) in progs.iter() {
                    if name.starts_with("fortinet") {
                        let _ = prog.resolve(&mut ctx);
                    }
                }
                Ok(target.value)
            }
            Vendor::CiscoAsa => {
                for (name, prog) in progs.iter() {
                    if name.starts_with("cisco") {
                        let _ = prog.resolve(&mut ctx);
                    }
                }
                Ok(target.value)
            }
            Vendor::PaloAlto => {
                for (name, prog) in progs.iter() {
                    if name.starts_with("palo") {
                        let _ = prog.resolve(&mut ctx);
                    }
                }
                Ok(target.value)
            }
            Vendor::Unknown => {
                let mut matched = false;
                for (name, prog) in progs.iter() {
                    if !name.starts_with("fortinet") && !name.starts_with("cisco") && !name.starts_with("palo") {
                        let mut test_map = BTreeMap::new();
                        test_map.insert("message".into(), Value::from(raw_log));
                        let mut test_target = TargetValue {
                            value: Value::Object(test_map),
                            metadata: Value::Object(BTreeMap::new()),
                            secrets: Secrets::new(),
                        };
                        let mut test_state = RuntimeState::default();
                        let mut test_ctx = Context::new(&mut test_target, &mut test_state, &tz);
                        if prog.resolve(&mut test_ctx).is_ok() {
                            if let Value::Object(ref obj) = test_target.value {
                                let has_domain_fields = obj.keys().any(|k| {
                                    k != "message" && k != "class_uid" && k != "category_uid" && k != "type_uid"
                                });
                                if obj.contains_key("class_uid") && has_domain_fields {
                                    target = test_target;
                                    matched = true;
                                    break;
                                }
                            }
                        }
                    }
                }
                if matched {
                    Ok(target.value)
                } else {
                    bail!("Unknown vendor, cannot parse")
                }
            }
        }
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
