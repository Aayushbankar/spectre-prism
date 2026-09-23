import os
import yaml

def load_config(config_path="prism-brain/config.yaml"):
    default_config = {
        "device": "cpu",
        "triage": {"engine": "heuristic", "model": "open-jev-deberta-v3-large"},
        "coder": {"enabled": False, "host": "http://localhost:11434", "model": "llama3", "timeout": 2},
        "watcher": {"path": "/var/run/prism/dlq.jsonl", "fallback": "/tmp/prism/dlq.jsonl"}
    }
    
    # Try looking in current directory (e.g. if running tests from inside prism-brain)
    paths_to_try = [config_path, "config.yaml"]
    loaded_data = None
    for p in paths_to_try:
        if os.path.exists(p):
            with open(p, "r") as f:
                loaded_data = yaml.safe_load(f)
                break
                
    if loaded_data:
        # Merge dicts safely (shallow merge is fine here)
        for k, v in loaded_data.items():
            if isinstance(v, dict) and isinstance(default_config.get(k), dict):
                default_config[k].update(v)
            else:
                default_config[k] = v
                
    # Env overrides
    if "PRISM_DEVICE" in os.environ:
        default_config["device"] = os.environ["PRISM_DEVICE"]
        
    return default_config
