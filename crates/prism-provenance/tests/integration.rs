use flume;
use prism_common::{RawEvent, ProvenanceMeta, LogSource};
use prism_provenance::ticker::IntegrityTicker;
use prism_provenance::audit::audit_vault_file;
use std::time::Duration;
use chrono::Utc;
use bytes::Bytes;
use std::fs;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use parquet::arrow::ArrowWriter;
use arrow::array::{BinaryArray, StringArray, Int64Array};
use arrow::record_batch::RecordBatch;
use std::sync::Arc;
use arrow::array::{BinaryBuilder, Int64Builder, StringBuilder};
use arrow::datatypes::{DataType, Field, Schema};
use std::fs::File;

#[tokio::test]
async fn test_integrity_plane_success() {
    let output_dir = "test_vault_success";
    fs::remove_dir_all(output_dir).ok();
    
    let (tx, rx) = flume::bounded(100);
    // Use a batch size of 10 so it flushes after 10 rows
    let ticker = IntegrityTicker::new(rx, output_dir, 10, Duration::from_millis(500)).unwrap();
    
    let ticker_handle = tokio::spawn(async move {
        ticker.run().await.unwrap();
    });
    
    let mut expected_tree = prism_provenance::merkle::ProvenanceTree::new();
    
    // Feed 10 heterogeneous logs
    let payloads = vec![
        b"syslog: connection refused".to_vec(),
        b"nginx: GET /index.html 200".to_vec(),
        b"{\"event\": \"login\", \"user\": \"admin\"}".to_vec(),
        b"kernel: out of memory".to_vec(),
        b"auth: PAM session opened".to_vec(),
        b"cisco-asa: Teardown TCP connection".to_vec(),
        b"fortigate: webfilter block".to_vec(),
        b"paloalto: threat detected".to_vec(),
        b"apache: 404 not found".to_vec(),
        b"mysql: Access denied for user".to_vec(),
    ];
    
    for (i, p) in payloads.into_iter().enumerate() {
        let hash = blake3::hash(&p);
        expected_tree.push_leaf(&hash);
        
        let source = if i % 2 == 0 { LogSource::Udp(std::net::SocketAddr::from(([127, 0, 0, 1], 514))) } else { LogSource::Unknown };
        
        let event = RawEvent {
            payload: Bytes::from(p),
            metadata: ProvenanceMeta {
                hash,
                timestamp: Utc::now(),
                source,
            }
        };
        tx.send(event).unwrap();
    }
    
    let expected_root = expected_tree.root_hash().unwrap();
    
    // Drop the channel to flush
    drop(tx);
    
    // Wait for the ticker to finish
    ticker_handle.await.unwrap();
    
    // At this point we should have a parquet file in test_vault_success.
    let paths = fs::read_dir(output_dir).unwrap();
    let mut parquet_files = vec![];
    for path in paths {
        let p = path.unwrap().path();
        if p.extension().and_then(|s| s.to_str()) == Some("parquet") {
            parquet_files.push(p);
        }
    }
    
    assert!(!parquet_files.is_empty());
    let parquet_file = &parquet_files[0];
    
    // Audit the file and expect success
    let root_hash = audit_vault_file(parquet_file.to_str().unwrap()).unwrap();
    
    // Verify the root hash matches the expected 10-leaf tree root
    assert_eq!(hex::encode(root_hash), hex::encode(expected_root));
}

#[tokio::test]
async fn test_integrity_plane_mutation_fails() {
    let output_dir = "test_vault_mutate";
    fs::remove_dir_all(output_dir).ok();
    
    let (tx, rx) = flume::bounded(100);
    let ticker = IntegrityTicker::new(rx, output_dir, 1, Duration::from_millis(500)).unwrap();
    
    let ticker_handle = tokio::spawn(async move {
        ticker.run().await.unwrap();
    });
    
    // Feed real logs
    let raw_payload = b"Original safe payload".to_vec();
    let hash1 = blake3::hash(&raw_payload);
    let event1 = RawEvent {
        payload: Bytes::from(raw_payload.clone()),
        metadata: ProvenanceMeta {
            hash: hash1,
            timestamp: Utc::now(),
            source: LogSource::Unknown,
        }
    };
    
    tx.send(event1).unwrap();
    drop(tx);
    ticker_handle.await.unwrap();
    
    let paths = fs::read_dir(output_dir).unwrap();
    let mut parquet_files = vec![];
    for path in paths {
        let p = path.unwrap().path();
        if p.extension().and_then(|s| s.to_str()) == Some("parquet") {
            parquet_files.push(p);
        }
    }
    
    assert!(!parquet_files.is_empty());
    let original_parquet_file = &parquet_files[0];
    
    // Read the file and rewrite it with a mutated payload
    let mut reader = ParquetRecordBatchReaderBuilder::try_new(File::open(original_parquet_file).unwrap()).unwrap().build().unwrap();
    let batch = reader.next().unwrap().unwrap();
    
    let ts_col = batch.column(0).as_any().downcast_ref::<Int64Array>().unwrap();
    let src_col = batch.column(1).as_any().downcast_ref::<StringArray>().unwrap();
    let hash_col = batch.column(2).as_any().downcast_ref::<StringArray>().unwrap();
    let _payload_col = batch.column(3).as_any().downcast_ref::<BinaryArray>().unwrap();
    
    let schema = Arc::new(Schema::new(vec![
        Field::new("timestamp_ms", DataType::Int64, false),
        Field::new("source", DataType::Utf8, false),
        Field::new("hash", DataType::Utf8, false),
        Field::new("payload", DataType::Binary, false),
    ]));
    
    let mut ts_builder = Int64Builder::new();
    let mut src_builder = StringBuilder::new();
    let mut hash_builder = StringBuilder::new();
    let mut payload_builder = BinaryBuilder::new();
    
    // We only have 1 row
    ts_builder.append_value(ts_col.value(0));
    src_builder.append_value(src_col.value(0));
    hash_builder.append_value(hash_col.value(0));
    
    // MUTATION: We modify the payload but keep the hash the same
    let mutated_payload = b"Malicious mutated payload".to_vec();
    payload_builder.append_value(&mutated_payload);
    
    let mutated_batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(ts_builder.finish()),
            Arc::new(src_builder.finish()),
            Arc::new(hash_builder.finish()),
            Arc::new(payload_builder.finish()),
        ],
    ).unwrap();
    
    let mutated_file_path = format!("{}/mutated.parquet", output_dir);
    let mut writer = ArrowWriter::try_new(File::create(&mutated_file_path).unwrap(), schema, None).unwrap();
    writer.write(&mutated_batch).unwrap();
    writer.close().unwrap();
    
    // Delete the original so we don't accidentally test it
    fs::remove_file(original_parquet_file).unwrap();
    
    // Audit the mutated file and expect failure
    let result = audit_vault_file(&mutated_file_path);
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("Audit Failed: Payload hash mismatch"));
}
