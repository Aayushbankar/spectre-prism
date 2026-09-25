# PRISM Brain

The `prism-brain` Python module serves as the Control Plane, executing AI-driven parsing, clustering, and Human-in-the-Loop (HitL) operations.

## Key Features

- **`watcher.py`**: Employs an inotify/watchdog dual-path approach to monitor the DLQ for new unparsed logs.
- **`cluster.py`**: Uses Drain3 for O(n) streaming log template mining (clustering 52k logs).
- **`triage.py`**: Executes Laya (System1 ModernBERT 421M, Apache 2.0) for rapid AI classification, falling back to CPU heuristics when necessary.
- **`coder.py`**: Invokes llama-server with Q4 GGUF weights for sub-second Vector Remap Language (VRL) rule generation for new templates.
- **`gatekeeper.py`**: Implements the HitL rule validation and hot-reloading pipeline. Approved rules are written directly to `/etc/prism/rules`.

## Configuration
Update `config.yaml` to run in CPU-only mode:
```yaml
device: cpu
coder.enabled: false
```

## Testing
To run the brain tests:
```bash
PRISM_DEVICE=cpu pytest prism-brain -v
```
