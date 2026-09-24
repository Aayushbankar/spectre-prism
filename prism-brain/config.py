import os
import yaml
from pathlib import Path

def load_config(config_path=None):
    default_config = {
        "device": "cpu",
        "triage": {"engine": "heuristic", "model": "open-jev-deberta-v3-large"},
        "coder": {"enabled": False, "host": "http://localhost:11434", "model": "llama3", "timeout": 2},
        "watcher": {"path": "/var/run/prism/dlq.jsonl", "fallback": "/tmp/prism/dlq.jsonl"}
    }
    
    if config_path is None:
        config_path = str(Path(__file__).resolve().parent.parent / "prism-brain/config.yaml")

    if os.path.exists(config_path):
        with open(config_path, "r") as f:
            loaded_data = yaml.safe_load(f)
            
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
