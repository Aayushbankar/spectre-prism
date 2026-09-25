# HOW-TO: Control Plane

This guide covers running the Python-based AI parsing pipeline.

## 1. Running in CPU-Only Mode
If you do not have a GPU, configure `prism-brain` to use CPU mode.
Edit `config.yaml`:
```yaml
device: cpu
coder.enabled: false
```
Run the test suite:
```bash
PRISM_DEVICE=cpu pytest prism-brain -v
```

## 2. AI Clustering and Triage
The Control Plane monitors the DLQ. It pipes new logs to `cluster.py` which uses the Drain3 algorithm to cluster up to 52,000 logs into templates in O(n) time.
These templates are triaged by `triage.py` using the Laya (ModernBERT 421M) model.

## 3. Rule Generation and HitL
For new templates, `coder.py` invokes a quantized (Q4) LLM via `llama-server` to generate VRL rules.
These rules are validated by `gatekeeper.py`, and once approved by a Human-in-the-Loop, they are hot-reloaded into `/etc/prism/rules/` without restarting the data plane.
