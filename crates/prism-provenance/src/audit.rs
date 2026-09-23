use std::fs::File;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use arrow::array::{BinaryArray, StringArray};
use anyhow::{Result, bail};
use crate::merkle::ProvenanceTree;

pub fn audit_vault_file(path: &str) -> Result<[u8; 32]> {
    let file = File::open(path)?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)?;
    let reader = builder.build()?;
    
    let mut tree = ProvenanceTree::new();

    for batch_result in reader {
        let batch = batch_result?;
        
        // Ensure schema columns match vault.rs: 0: timestamp, 1: source, 2: hash, 3: payload
        let hash_col = batch.column(2).as_any().downcast_ref::<StringArray>().unwrap();
        let payload_col = batch.column(3).as_any().downcast_ref::<BinaryArray>().unwrap();
        
        for i in 0..batch.num_rows() {
            let recorded_hash_hex = hash_col.value(i);
            let payload_bytes = payload_col.value(i);
            
            // Add to Merkle Tree by parsing first
            let hash = blake3::Hash::from_hex(recorded_hash_hex).map_err(|e| anyhow::anyhow!("Invalid hex hash: {}", e))?;
            
            // Recompute hash
            let computed_hash = blake3::hash(payload_bytes);
            if computed_hash != hash {
                bail!("Audit Failed: Payload hash mismatch at row {}. Expected {}, got {}", i, recorded_hash_hex, computed_hash.to_hex());
            }
            
            tree.push_leaf(&hash)?;
        }
    }
    
    if let Some(root) = tree.root_hash() {
        Ok(root)
    } else {
        bail!("Empty vault file")
    }
}
