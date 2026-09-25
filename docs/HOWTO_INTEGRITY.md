# HOW-TO: Integrity Plane (Plane 4)

This runbook explains how to manage cold storage and cryptographic integrity in PRISM. This plane ensures Section 65B (Indian Evidence Act) compliance.

## 1. Inspecting the Cold Vault
Logs are flushed into Apache Parquet format. By default, the output directory is `./output_dir/`.

To inspect the parquet files, you can use standard data science tools like Python's `pandas` or CLI tools like `parquet-tools`:
```bash
parquet-tools inspect ./output_dir/*.parquet
```
Ensure you observe that columns are compressed using `ZSTD` (Zstandard).

## 2. Understanding `ledger.log`
Alongside the parquet files, the Integrity Plane maintains `output_dir/ledger.log`.
- Every batch of logs (up to 1<<16 logs) generates a single Merkle Root Hash.
- This hash is atomically written to `ledger.log` using an `fsync` (`sync_all()`) system call.
- This ensures that even on sudden power loss, cryptographic hashes of the prior blocks are safely on disk.

## 3. Auditing for Tampering
If you suspect data tampering or bit-rot, PRISM provides an audit function.

Run the integrity test suite to see the auditor in action:
```bash
cargo test -p prism-provenance test_integrity_plane_success
```
This function reads the entire Parquet file, re-hashes every single log line (`blake3`), reconstructs the Merkle Tree in memory, and compares the final root against `ledger.log`.

## 4. Simulating Tampering
You can run the failure test to see what happens when data is tampered:
```bash
cargo test -p prism-provenance test_integrity_plane_mutation_fails
```
This test artificially modifies a single byte in the Parquet payload, which cascades up the Merkle tree, causing the root hash to mismatch and instantly throwing a `"Payload hash mismatch"` security error.
