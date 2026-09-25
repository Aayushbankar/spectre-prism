# `prism-common`

The `prism-common` crate defines shared data structures, constants, and utilities used across all PRISM planes.

## Key Components

- **`RawEvent`**: The fundamental log event struct. Supports `utf8` and `b64` encoding.
- **`ProvenanceMeta`**: Tracks the origin and routing metadata of each event.
- **`LogSource`**: Enumeration of supported log origins (e.g., Firewall, Switch, OS).
- **`OcsfNetworkActivity`**: Rust representation of the Open Cybersecurity Schema Framework (OCSF) Network Activity class (category 4, class 4001).
- **`Bytes`**: Zero-copy byte representation for high-throughput zero-allocation passing.
- **Blake3 Hashing**: Fast cryptographic hashing of events for deduplication and integrity tracking.

## Usage
Provides serialization and deserialization roundtrips that are highly optimized and zero-copy where possible.
