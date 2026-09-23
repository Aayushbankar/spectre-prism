use std::sync::Arc;
use arrow::array::{BinaryBuilder, Int64Builder, StringBuilder};
use arrow::record_batch::RecordBatch;
use arrow::datatypes::{DataType, Field, Schema};
use parquet::arrow::ArrowWriter;
use parquet::file::properties::{WriterProperties, WriterVersion};
use parquet::basic::{Compression, ZstdLevel};
use std::fs::{File, create_dir_all};
use std::path::Path;
use prism_common::RawEvent;
use anyhow::Result;

pub struct VaultWriter {
    schema: Arc<Schema>,
    timestamp_builder: Int64Builder,
    source_builder: StringBuilder,
    hash_builder: StringBuilder,
    payload_builder: BinaryBuilder,
    batch_size: usize,
    output_dir: String,
    current_batch: usize,
    file_counter: u64,
}

impl VaultWriter {
    pub fn new(output_dir: &str, batch_size: usize) -> Result<Self> {
        let schema = Arc::new(Schema::new(vec![
            Field::new("timestamp_ms", DataType::Int64, false),
            Field::new("source", DataType::Utf8, false),
            Field::new("hash", DataType::Utf8, false),
            Field::new("payload", DataType::Binary, false),
        ]));

        if !Path::new(output_dir).exists() {
            create_dir_all(output_dir)?;
        }

        Ok(Self {
            schema,
            timestamp_builder: Int64Builder::new(),
            source_builder: StringBuilder::new(),
            hash_builder: StringBuilder::new(),
            payload_builder: BinaryBuilder::new(),
            batch_size,
            output_dir: output_dir.to_string(),
            current_batch: 0,
            file_counter: 0,
        })
    }

    pub fn append(&mut self, event: &RawEvent) -> Result<bool> {
        self.timestamp_builder.append_value(event.metadata.timestamp.timestamp_millis());
        let source_str = serde_json::to_string(&event.metadata.source).unwrap_or_else(|_| "{}".to_string());
        self.source_builder.append_value(source_str);
        self.hash_builder.append_value(event.metadata.hash.to_hex().as_str());
        self.payload_builder.append_value(event.payload.as_ref());

        self.current_batch += 1;
        if self.current_batch >= self.batch_size {
            self.flush()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn flush(&mut self) -> Result<()> {
        if self.current_batch == 0 {
            return Ok(());
        }
        
        let batch = RecordBatch::try_new(
            self.schema.clone(),
            vec![
                Arc::new(self.timestamp_builder.finish()),
                Arc::new(self.source_builder.finish()),
                Arc::new(self.hash_builder.finish()),
                Arc::new(self.payload_builder.finish()),
            ],
        )?;

        let props = WriterProperties::builder()
            .set_compression(Compression::ZSTD(ZstdLevel::default()))
            .set_writer_version(WriterVersion::PARQUET_2_0)
            .build();

        let filename = format!("{}/{:05}_{}.parquet", self.output_dir, self.file_counter, uuid::Uuid::new_v4());
        let file = File::create(&filename)?;
        self.file_counter += 1;
        
        let mut writer = ArrowWriter::try_new(file, self.schema.clone(), Some(props))?;
        writer.write(&batch)?;
        writer.close()?;

        self.current_batch = 0;
        Ok(())
    }
}
