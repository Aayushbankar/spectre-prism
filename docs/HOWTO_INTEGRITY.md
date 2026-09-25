# HOW-TO: Integrity Plane

This guide demonstrates how PRISM ensures log non-repudiation and legal compliance.

## 1. Writing Cold Storage Batches
PRISM automatically chunks incoming logs and compresses them into Parquet 2.0 files using Zstandard (ZSTD) compression across all columns. These files are stored in the `ColdVault`.

## 2. Auditing with Merkle Trees
Every batch is hashed using Blake3, and a Merkle Tree is constructed using the `rs_merkle` crate (capped at `1<<16` leaves).
The Merkle root is securely synchronized to disk:
```bash
cat /var/lib/prism/ledger.log
```
This ensures compliance with Section 65B of the Indian Evidence Act.

## 3. Verifying Bit-Rot
Run the integrity tests to simulate a verification workflow:
```bash
cargo test -p prism-provenance test_integrity_plane_success
```
This reads the Parquet files and verifies the hashes against `ledger.log` using the `audit.rs` module.
