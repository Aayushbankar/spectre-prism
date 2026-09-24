use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use std::path::PathBuf;
use std::collections::VecDeque;

pub struct TuiState {
    pub eps: Arc<AtomicU64>,
    pub eps_history: VecDeque<u64>,
    pub dlq_path: PathBuf,
    pub ledger_path: PathBuf,
    pub rules_dir: PathBuf,
    pub dlq_items: Vec<(String, String, String)>,
    pub ledger_history: Vec<(f64, f64)>,
    pub hitl_rules: Vec<String>,
    pub tick_count: f64,
    pub ledger_line_count: u64,
    pub ledger_tail: String,
}

impl Default for TuiState {
    fn default() -> Self {
        Self {
            eps: Arc::new(AtomicU64::new(0)),
            eps_history: VecDeque::with_capacity(100),
            dlq_path: PathBuf::from("/var/run/prism/dlq.jsonl"),
            ledger_path: PathBuf::from("/tmp/test_vault_success/ledger.log"),
            rules_dir: PathBuf::from("/etc/prism/rules"),
            dlq_items: Vec::new(),
            ledger_history: Vec::new(),
            hitl_rules: Vec::new(),
            tick_count: 0.0,
            ledger_line_count: 0,
            ledger_tail: String::new(),
        }
    }
}

impl TuiState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_paths(dlq_path: PathBuf, ledger_path: PathBuf, rules_dir: PathBuf) -> Self {
        Self {
            dlq_path,
            ledger_path,
            rules_dir,
            ..Self::default()
        }
    }
}
