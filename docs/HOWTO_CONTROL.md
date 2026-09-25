# HOW-TO: Control Plane (Plane 3)

The Python-based Control Plane handles alien log triaging and rule generation.

## 1. Running in CPU-Only Mode
For edge devices lacking GPU capability, PRISM gracefully downgrades to a CPU heuristic engine rather than spinning up heavy models.

To run the test suite mimicking a CPU-only environment:
```bash
PRISM_DEVICE=cpu pytest prism-brain/tests -v
```
This triggers `test_heuristic_cpu` which verifies the triage falls back to a deterministic <1ms latency path rather than utilizing Laya.

## 2. Testing Drain3 Clustering
Drain3 processes massive volumes of alien logs into fixed-depth trees.
To observe Drain3 clustering 50,000 logs into just a few templates:
```bash
pytest prism-brain/tests/test_drain_isolated.py::test_drain_50k_bigdata -s
```
You will see output showing the `LogClusterer` parsing logs at over 80,000 EPS.

## 3. Laya Model Inference
If you have a GPU or sufficient CPU, you can enable semantic classification using Laya (ModernBERT 421M).
Modify `prism-brain/config.yaml`:
```yaml
device: cuda
triage: laya
```
This maps templates to specific types (Firewall, Web Proxy) with an ECE of 0.081.

## 4. Testing Rule Generation (Coder) & Hot Reloading
When an alien log is classified, a new VRL rule is generated and pushed to `/etc/prism/rules`.

To test the watchdog (`inotify`) pipeline all the way to hot-reloading:
```bash
pytest prism-brain/tests/test_watcher_gatekeeper_isolated.py::test_gatekeeper_hot_reload
```
The Gatekeeper writes the file, which instantly prompts the Rust Data Plane to reload its internal VRL structures without dropping packets.
