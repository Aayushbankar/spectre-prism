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
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::basic::Compression;

#[tokio::test]
async fn test_integrity_plane_success() {
    let output_dir = "test_vault_success";
    fs::remove_dir_all(output_dir).ok();
    
    let (tx, rx) = flume::bounded(100);
    // Use a batch size of 1 so it tests multi-file vault writes per row
    let ticker = IntegrityTicker::new(rx, output_dir, 1, Duration::from_millis(500)).unwrap();
    
    let ticker_handle = tokio::spawn(async move {
        ticker.run().await.unwrap();
    });
    
    let mut expected_tree = prism_provenance::merkle::ProvenanceTree::new();
    
    // Feed heterogeneous logs: real vendor samples
    let payloads = vec![
        b"date=2024-01-01 time=12:00:00 devname=\"FW01\" devid=\"FG100\" logid=\"0000000013\" type=\"traffic\" subtype=\"forward\" level=\"notice\" srcip=192.168.1.5 dstip=8.8.8.8 action=\"accept\"".to_vec(),
        b"%ASA-6-302013: Built inbound TCP connection 44439167 for outside:192.168.1.5/54321 (192.168.1.5/54321) to inside:10.0.0.1/80 (10.0.0.1/80)".to_vec(),
        b"1,2024/01/01 12:00:00,0011C1234,THREAT,vulnerability,1,2024/01/01 12:00:00,192.168.1.5,10.0.0.1,0.0.0.0,0.0.0.0,Rule1,user1,web-browsing,vsys1,trust,untrust,eth1/1,eth1/2,syslog,2024/01/01 12:00:00,54321,80,0,0,12345,0x400000,1,1,1,1,1,1,0,0,0,0,0,0,0,0,0".to_vec(),
    ];
    
    for p in payloads {
        let hash = blake3::hash(&p);
        expected_tree.push_leaf(&hash).unwrap();
        
        let event = RawEvent {
            payload: Bytes::from(p),
            metadata: ProvenanceMeta {
                hash,
                timestamp: Utc::now(),
                source: LogSource::Unknown,
            }
        };
        tx.send(event).unwrap();
    }
    
    let expected_root = expected_tree.root_hash().unwrap();
    
    drop(tx);
    ticker_handle.await.unwrap();
    
    // Assert ledger file exists and contains the root hash
    let ledger_path = format!("{}/ledger.log", output_dir);
    assert!(fs::metadata(&ledger_path).is_ok(), "ledger.log must exist");
    let ledger_contents = fs::read_to_string(&ledger_path).unwrap();
    assert!(ledger_contents.contains(&hex::encode(expected_root)));

    let paths = fs::read_dir(output_dir).unwrap();
    let mut parquet_files = vec![];
    for path in paths {
        let p = path.unwrap().path();
        if p.extension().and_then(|s| s.to_str()) == Some("parquet") {
            parquet_files.push(p);
        }
    }
    
    // We expect 3 parquet files because batch_size=1 and we pushed 3 events.
    assert_eq!(parquet_files.len(), 3);
    
    // Audit the first file and verify Zstd compression via WriterProperties
    let parquet_file = &parquet_files[0];
    let file = File::open(&parquet_file).unwrap();
    let reader = SerializedFileReader::new(file).unwrap();
    let metadata = reader.metadata();
    let row_group = metadata.row_group(0);
    let column_chunk = row_group.column(0);
    match column_chunk.compression() {
        Compression::ZSTD(_) => {},
        other => panic!("Expected ZSTD compression, got {:?}", other),
    }
}

#[tokio::test]
async fn test_empty_vault() {
    let output_dir = "test_vault_empty";
    fs::remove_dir_all(output_dir).ok();
    
    let (tx, rx) = flume::bounded(100);
    let ticker = IntegrityTicker::new(rx, output_dir, 1, Duration::from_millis(500)).unwrap();
    
    let ticker_handle = tokio::spawn(async move {
        ticker.run().await.unwrap();
    });
    
    drop(tx);
    ticker_handle.await.unwrap();
    
    // Vault should be empty, no parquet, no ledger log
    let paths = fs::read_dir(output_dir).unwrap();
    assert_eq!(paths.count(), 0);
}

#[tokio::test]
async fn test_merkle_tree_limit() {
    let mut tree = prism_provenance::merkle::ProvenanceTree::new();
    let dummy_hash = blake3::hash(b"dummy");
    for _ in 0..65536 {
        tree.push_leaf(&dummy_hash).unwrap();
    }
    // The next one should fail
    let res = tree.push_leaf(&dummy_hash);
    assert!(res.is_err());
    assert_eq!(res.unwrap_err().to_string(), "16-level limit reached");
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
