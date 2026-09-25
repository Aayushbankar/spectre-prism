# `prism-provenance`

The `prism-provenance` crate provides the Integrity and Vault planes, responsible for cold storage and cryptographic auditing.

## Key Features

- **`ColdVault`**: Manages long-term storage of raw log data.
- **Parquet 2.0 with Zstandard (ZSTD)**: Compresses all columns using ZSTD for maximum storage efficiency, making it feasible for 180-day retention policies.
- **Merkle Tree Auditing**: Uses `rs_merkle` to construct cryptographic proofs of log integrity. Caps trees at `1<<16` leaves.
- **Legal Admissibility**: Ensures compliance with Section 65B of the Indian Evidence Act via non-repudiable logs.
- **`ledger.log` Synchronization**: Persists Merkle roots to disk securely with `sync_all`.
- **`audit.rs`**: Verification module for checking bit-rot or tampering.

## Testing
To run the integrity tests:
```bash
cargo test -p prism-provenance
```
