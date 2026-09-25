# `prism-tui`

The `prism-tui` crate provides the Presentation Plane dashboard directly in the terminal, giving real-time visibility into the PRISM system.

## Key Features

- **Ratatui 0.30 & Crossterm 0.29**: Built on robust Rust terminal UI libraries.
- **Tokio Loop**: Runs a 100ms interval polling loop.
- **Low Overhead Polling**: Uses seek-based file polling to read metrics without high CPU usage.

## 4-Pane Engine Room
1. **Pane 1 (EPS Gauge & Sparkline)**: Visualizes current Events Per Second throughput.
2. **Pane 2 (DLQ Table)**: Lists recent failures or alien logs routed to the Dead Letter Queue.
3. **Pane 3 (Merkle Ticker & Line Chart)**: Displays real-time cryptographic hashing and integrity plane commits.
4. **Pane 4 (HitL Gatekeeper List)**: Shows pending and approved AI-generated VRL rules.

## Testing & Execution
To run the TUI:
```bash
cargo run -p prism-tui
```
For headless tests:
```bash
cargo test -p prism-tui
```
