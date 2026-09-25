# `prism-provenance`

The `prism-provenance` crate implements the Integrity/Vault Plane (Plane 4) of the PRISM framework. It handles immutable log archiving and provides mathematical proofs of data integrity, directly fulfilling Section 65B of the Indian Evidence Act for digital records admissibility.

## Key Features

### 1. Parquet Cold Storage (`ColdVault`)
All ingested logs are continuously flushed to disk in Apache Parquet 2.0 format.
- Uses Zstandard (ZSTD) compression across all columns for extreme space efficiency.
- Preserves exact byte-for-byte fidelity of the original payload.
- Achieves high write-throughput via batching before writing to disk.

### 2. Merkle Tree Integrity Auditing
Every log line is hashed upon ingestion. These hashes form the leaves of a Merkle tree.
- Uses `rs_merkle` to maintain a tree with a strict capacity of `1<<16` (65,536 logs) per batch.
- Generates a singular Merkle root hash for the entire batch.
- Protects against bit-rot, accidental modifications, or malicious tampering.

### 3. Ledger Synchronization (`ledger.log`)
The Merkle roots are atomically appended to a write-ahead `ledger.log`.
- Calls `sync_all` (fsync) to guarantee the hash is persisted to non-volatile storage.
- A 500ms ticker continuously flushes roots.

### 4. Admissibility and Auditing (`audit.rs`)
Provides utilities to traverse the cold storage vaults, re-hash the payloads, and verify them against the stored Merkle roots in the ledger. Any discrepancy instantly flags the archive as tampered.

## How to Test
Execute the integrity tests via Cargo:

```bash
cargo test -p prism-provenance
```

This runs tests like `test_integrity_plane_success`, which simulates writing 10 heterogeneous logs to Parquet, computing the Merkle roots, and auditing the result perfectly.
