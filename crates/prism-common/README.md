# `prism-common`

The `prism-common` crate defines the shared data structures, constants, and highly optimized utilities used across all PRISM planes. It acts as the backbone for inter-plane communication and ensuring high-performance zero-copy operations.

## Key Components

### 1. `RawEvent`
The fundamental log event struct. It supports both `utf8` and `b64` encoded payloads natively. The payload itself is wrapped in an optimized byte buffer to minimize allocations.
- Handles parsing of incoming payloads.
- Maintains strict UTF-8 boundaries.

### 2. `ProvenanceMeta`
Tracks the origin and routing metadata of each event. Crucial for establishing the chain of custody from ingestion to cold storage, meeting strict admissibility requirements.

### 3. `LogSource`
Enumeration of supported log origins. Categorizes inputs such as Firewall, Switch, OS, and generic UDP traffic to simplify routing.

### 4. `OcsfNetworkActivity`
The Rust representation of the Open Cybersecurity Schema Framework (OCSF) Network Activity class (category 4, class 4001). All logs normalized by the Data Plane end up mapped to this structure.

### 5. `Bytes`
A zero-copy byte representation for high-throughput zero-allocation passing. It prevents unnecessary `clone()` calls when data moves between the ingest channel, the router, and the cold storage writer.

### 6. Blake3 Hashing
Fast cryptographic hashing (`blake3`) is applied immediately upon ingestion. The resulting hash is used for:
- Deduplication.
- Cryptographic Merkle tree integrity tracking in `prism-provenance`.

## Usage & Performance
`prism-common` provides serialization and deserialization roundtrips that are heavily micro-optimized. Tests (`test_raw_event_serialization_roundtrip`) ensure zero regressions in serialization speed, maintaining the sub-microsecond processing guarantee.
