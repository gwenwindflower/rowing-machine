//! Scenario-agnostic entity schemas and rows handed to format writers.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, ensure};

mod csv;
mod jsonl;
mod parquet;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum Format {
    #[default]
    Csv,
    Jsonl,
    Parquet,
}

impl Format {
    fn extension(self) -> &'static str {
        match self {
            Self::Csv => "csv",
            Self::Jsonl => "jsonl",
            Self::Parquet => "parquet",
        }
    }
}

struct EntityOutput {
    schema: EntitySchema,
    key_indices: Vec<usize>,
    keys: HashSet<PrimaryKey>,
    writer: Option<Box<dyn EntityWriter>>,
    count: u64,
    estimated_rows: usize,
}

pub struct OutputSink {
    directory: PathBuf,
    prefix: String,
    format: Format,
    compress: bool,
    entities: BTreeMap<&'static str, EntityOutput>,
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum KeyValue {
    Uuid(u128),
    Text(String),
    Integer(i64),
    Float(u64),
    Boolean(bool),
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum PrimaryKey {
    Uuid(u128),
    Composite(Vec<KeyValue>),
}

enum Payload {
    Bytes(Vec<u8>),
    Arrow(arrow::record_batch::RecordBatch),
}

struct PreparedEntity {
    keys: Vec<PrimaryKey>,
    payload: Payload,
}

pub(crate) struct PreparedUnit(Vec<(&'static str, PreparedEntity)>);

pub(crate) struct UnitEncoder {
    entities: BTreeMap<&'static str, (EntitySchema, Vec<usize>)>,
    format: Format,
}

impl UnitEncoder {
    pub(crate) fn new(schemas: Vec<EntitySchema>, format: Format) -> Self {
        Self {
            entities: schemas
                .into_iter()
                .map(|schema| {
                    let indices = schema
                        .primary_key
                        .iter()
                        .map(|name| {
                            schema
                                .columns
                                .iter()
                                .position(|column| column.name == *name)
                                .expect("sink validated schema")
                        })
                        .collect();
                    (schema.name, (schema, indices))
                })
                .collect(),
            format,
        }
    }

    pub(crate) fn encode(&self, rows: &crate::scenario::UnitRows) -> Result<PreparedUnit> {
        let mut grouped = BTreeMap::<&'static str, Vec<&Row>>::new();
        for (entity, row) in rows {
            grouped.entry(entity).or_default().push(row);
        }
        grouped
            .into_iter()
            .map(|(entity, rows)| {
                let (schema, indices) = self
                    .entities
                    .get(entity)
                    .with_context(|| format!("unknown output entity {entity}"))?;
                Ok((entity, prepare(schema, indices, &rows, self.format)?))
            })
            .collect::<Result<Vec<_>>>()
            .map(PreparedUnit)
    }
}

fn prepare(
    schema: &EntitySchema,
    key_indices: &[usize],
    rows: &[&Row],
    format: Format,
) -> Result<PreparedEntity> {
    let mut keys = Vec::with_capacity(rows.len());
    for row in rows {
        ensure!(
            row.len() == schema.columns.len(),
            "entity {}: expected {} fields, got {}",
            schema.name,
            schema.columns.len(),
            row.len()
        );
        for (index, (column, value)) in schema.columns.iter().zip(row.iter()).enumerate() {
            let empty = matches!(value, Value::Null)
                || matches!(value, Value::Text(text) if text.is_empty());
            ensure!(
                !empty || (column.nullable && !key_indices.contains(&index)),
                "entity {}: column {} must not be empty",
                schema.name,
                column.name
            );
            ensure!(
                matches!(value, Value::Null) || value.column_type() == column.column_type,
                "entity {}: column {} has incorrect value type",
                schema.name,
                column.name
            );
            match value {
                Value::Uuid(bytes) => ensure!(
                    bytes[6] >> 4 == 4 && bytes[8] >> 6 == 2,
                    "UUID must have version 4 and RFC 4122 variant bits"
                ),
                Value::Float(number) => ensure!(number.is_finite(), "float must be finite"),
                Value::Date(days) => {
                    jiff::Timestamp::from_second(i64::from(*days) * 86_400)?;
                }
                Value::Timestamp(micros) => {
                    jiff::Timestamp::from_microsecond(*micros)?;
                }
                _ => {}
            }
        }
        let key = if let [index] = key_indices
            && let Value::Uuid(bytes) = row[*index]
        {
            PrimaryKey::Uuid(u128::from_be_bytes(bytes))
        } else {
            PrimaryKey::Composite(
                key_indices
                    .iter()
                    .map(|&index| match &row[index] {
                        Value::Uuid(bytes) => KeyValue::Uuid(u128::from_be_bytes(*bytes)),
                        Value::Text(text) => KeyValue::Text(text.clone()),
                        Value::Integer(value) | Value::Cents(value) | Value::Timestamp(value) => {
                            KeyValue::Integer(*value)
                        }
                        Value::Date(value) => KeyValue::Integer(i64::from(*value)),
                        Value::Float(value) => {
                            KeyValue::Float(if *value == 0.0 { 0 } else { value.to_bits() })
                        }
                        Value::Boolean(value) => KeyValue::Boolean(*value),
                        Value::Null => unreachable!("primary key validated"),
                    })
                    .collect(),
            )
        };
        keys.push(key);
    }
    let payload = match format {
        Format::Csv => Payload::Bytes(csv::encode(schema, rows)?),
        Format::Jsonl => Payload::Bytes(jsonl::encode(schema, rows)?),
        Format::Parquet => Payload::Arrow(parquet::encode(schema, rows)?),
    };
    Ok(PreparedEntity { keys, payload })
}

impl OutputSink {
    /// Creates a CSV sink with files opened on the first row of each entity.
    ///
    /// # Errors
    /// Returns an error for invalid or duplicate entity schemas.
    pub fn new(directory: &Path, prefix: &str, schemas: Vec<EntitySchema>) -> Result<Self> {
        Self::with_format(directory, prefix, schemas, Format::Csv)
    }

    /// Creates a sink whose entity files open on their first row.
    ///
    /// # Errors
    /// Returns an error for invalid or duplicate entity schemas.
    pub fn with_format(
        directory: &Path,
        prefix: &str,
        schemas: Vec<EntitySchema>,
        format: Format,
    ) -> Result<Self> {
        Self::with_options(directory, prefix, schemas, format, false)
    }

    /// Creates a sink with format-specific compression and lazy entity files.
    ///
    /// # Errors
    /// Returns an error for invalid schemas or unsupported CSV compression.
    pub fn with_options(
        directory: &Path,
        prefix: &str,
        schemas: Vec<EntitySchema>,
        format: Format,
        compress: bool,
    ) -> Result<Self> {
        ensure!(
            !compress || format != Format::Csv,
            "--compress cannot be used with --format csv; choose --format jsonl or --format parquet"
        );
        let mut entities = BTreeMap::new();
        for schema in schemas {
            ensure!(
                !schema.columns.is_empty(),
                "entity {} has no columns",
                schema.name
            );
            let mut names = BTreeSet::new();
            for column in &schema.columns {
                ensure!(
                    names.insert(column.name),
                    "entity {} has duplicate column {}",
                    schema.name,
                    column.name
                );
            }
            ensure!(
                !schema.primary_key.is_empty(),
                "entity {} has no primary key",
                schema.name
            );
            let key_indices = schema
                .primary_key
                .iter()
                .map(|name| {
                    schema
                        .columns
                        .iter()
                        .position(|column| column.name == *name)
                        .with_context(|| {
                            format!(
                                "entity {} has unknown primary key column {name}",
                                schema.name
                            )
                        })
                })
                .collect::<Result<Vec<_>>>()?;
            let name = schema.name;
            ensure!(
                !entities.contains_key(name),
                "duplicate entity schema {name}"
            );
            entities.insert(
                name,
                EntityOutput {
                    schema,
                    key_indices,
                    keys: HashSet::new(),
                    writer: None,
                    count: 0,
                    estimated_rows: 0,
                },
            );
        }
        Ok(Self {
            directory: directory.to_owned(),
            prefix: prefix.to_owned(),
            format,
            compress,
            entities,
        })
    }

    /// Sets an advisory row estimate before an entity's writer opens.
    pub fn estimate_rows(&mut self, entity: &str, rows: usize) {
        if let Some(output) = self.entities.get_mut(entity)
            && output.writer.is_none()
        {
            output.estimated_rows = rows;
        }
    }

    /// Validates and writes one row in schema column order.
    ///
    /// # Errors
    /// Returns an error for malformed rows, duplicate keys, or failed file writes.
    pub fn write(&mut self, entity: &str, row: &Row) -> Result<()> {
        let output = self
            .entities
            .get(entity)
            .with_context(|| format!("unknown output entity {entity}"))?;
        let prepared = prepare(&output.schema, &output.key_indices, &[row], self.format)?;
        self.write_prepared(output.schema.name, prepared, 1)
    }

    pub(crate) fn write_unit(&mut self, unit: PreparedUnit, remaining_units: usize) -> Result<()> {
        for (entity, prepared) in unit.0 {
            self.write_prepared(entity, prepared, remaining_units)?;
        }
        Ok(())
    }

    fn write_prepared(
        &mut self,
        entity: &str,
        prepared: PreparedEntity,
        remaining_units: usize,
    ) -> Result<()> {
        let output = self
            .entities
            .get_mut(entity)
            .with_context(|| format!("unknown output entity {entity}"))?;
        let count = prepared.keys.len();
        for key in prepared.keys {
            ensure!(
                output.keys.insert(key),
                "entity {entity}: duplicate primary key"
            );
        }
        if output.writer.is_none() {
            if output.estimated_rows == 0 {
                output.estimated_rows = count.saturating_mul(remaining_units);
            }
            std::fs::create_dir_all(&self.directory).with_context(|| {
                format!("creating output directory {}", self.directory.display())
            })?;
            let extension = if self.format == Format::Jsonl && self.compress {
                "jsonl.gz"
            } else {
                self.format.extension()
            };
            let path = self
                .directory
                .join(format!("{}_{entity}.{}", self.prefix, extension));
            output.writer = Some(match self.format {
                Format::Csv => Box::new(csv::CsvWriter::new(&path, &output.schema)?),
                Format::Jsonl => Box::new(jsonl::JsonlWriter::new(
                    &path,
                    &output.schema,
                    self.compress,
                )?),
                Format::Parquet => Box::new(parquet::ParquetWriter::new(
                    &path,
                    &output.schema,
                    output.estimated_rows,
                    self.compress,
                )?),
            });
        }
        let writer = output
            .writer
            .as_mut()
            .context("entity writer was not opened")?;
        match prepared.payload {
            Payload::Bytes(bytes) => writer.write_bytes(&bytes)?,
            Payload::Arrow(batch) => writer.write_batch(&batch)?,
        }
        output.count += count as u64;
        Ok(())
    }

    /// Flushes every created file and returns row counts for all declared entities.
    ///
    /// # Errors
    /// Returns an error when a file cannot be flushed.
    pub fn finish(self) -> Result<BTreeMap<String, u64>> {
        let mut counts = BTreeMap::new();
        for (name, output) in self.entities {
            if let Some(writer) = output.writer {
                writer.finish()?;
            }
            counts.insert(name.to_owned(), output.count);
        }
        Ok(counts)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnType {
    Uuid,
    Text,
    Integer,
    Cents,
    Float,
    Boolean,
    Date,
    Timestamp,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Column {
    pub name: &'static str,
    pub column_type: ColumnType,
    pub nullable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntitySchema {
    pub name: &'static str,
    pub columns: Vec<Column>,
    /// Column names forming the primary key, in order.
    pub primary_key: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Uuid([u8; 16]),
    Text(String),
    Integer(i64),
    Cents(i64),
    Float(f64),
    Boolean(bool),
    /// Days since the Unix epoch.
    Date(i32),
    /// Microseconds since the Unix epoch, UTC.
    Timestamp(i64),
}

impl Value {
    fn column_type(&self) -> ColumnType {
        match self {
            Self::Null | Self::Text(_) => ColumnType::Text,
            Self::Uuid(_) => ColumnType::Uuid,
            Self::Integer(_) => ColumnType::Integer,
            Self::Cents(_) => ColumnType::Cents,
            Self::Float(_) => ColumnType::Float,
            Self::Boolean(_) => ColumnType::Boolean,
            Self::Date(_) => ColumnType::Date,
            Self::Timestamp(_) => ColumnType::Timestamp,
        }
    }
}

pub type Row = Vec<Value>;

/// Writes the rows of one entity in one format.
pub trait EntityWriter: Send {
    /// Appends encoded CSV or JSONL records.
    ///
    /// # Errors
    /// Returns an error for unsupported encodings or failed writes.
    fn write_bytes(&mut self, _bytes: &[u8]) -> Result<()> {
        anyhow::bail!("writer does not accept encoded text")
    }

    /// Appends a typed Arrow batch to a Parquet file.
    ///
    /// # Errors
    /// Returns an error for unsupported encodings or failed writes.
    fn write_batch(&mut self, _batch: &arrow::record_batch::RecordBatch) -> Result<()> {
        anyhow::bail!("writer does not accept Arrow batches")
    }
    /// Appends one row, already validated against the entity schema.
    ///
    /// # Errors
    ///
    /// Returns an error when the underlying file cannot be written.
    fn write_row(&mut self, row: &Row) -> Result<()>;

    /// Flushes buffered rows and closes the file.
    ///
    /// # Errors
    ///
    /// Returns an error when the underlying file cannot be flushed or closed.
    fn finish(self: Box<Self>) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn schema() -> EntitySchema {
        EntitySchema {
            name: "records",
            columns: vec![
                Column {
                    name: "id",
                    column_type: ColumnType::Text,
                    nullable: false,
                },
                Column {
                    name: "sku",
                    column_type: ColumnType::Integer,
                    nullable: false,
                },
                Column {
                    name: "note",
                    column_type: ColumnType::Text,
                    nullable: true,
                },
            ],
            primary_key: vec!["id", "sku"],
        }
    }

    #[test]
    fn prepared_units_reject_duplicate_keys_across_units() {
        let directory = tempfile::tempdir().unwrap();
        let schemas = vec![schema()];
        let encoder = UnitEncoder::new(schemas.clone(), Format::Csv);
        let mut sink = OutputSink::new(directory.path(), "shop", schemas).unwrap();
        let rows = vec![(
            "records",
            vec![Value::Text("a".into()), Value::Integer(1), Value::Null],
        )];
        sink.write_unit(encoder.encode(&rows).unwrap(), 2).unwrap();
        assert!(
            sink.write_unit(encoder.encode(&rows).unwrap(), 1)
                .unwrap_err()
                .to_string()
                .contains("duplicate primary key")
        );
        assert_eq!(sink.finish().unwrap()["records"], 1);
    }

    #[test]
    fn worker_encoding_rejects_malformed_values_in_every_format() {
        for format in [Format::Csv, Format::Jsonl, Format::Parquet] {
            let encoder = UnitEncoder::new(vec![schema()], format);
            for row in [
                vec![],
                vec![Value::Null, Value::Integer(1), Value::Null],
                vec![
                    Value::Text("a".into()),
                    Value::Text("wrong".into()),
                    Value::Null,
                ],
            ] {
                assert!(encoder.encode(&vec![("records", row)]).is_err());
            }
        }
    }

    #[test]
    fn jsonl_preserves_schema_order_nulls_and_escaped_text() {
        let directory = tempfile::tempdir().unwrap();
        let mut sink =
            OutputSink::with_format(directory.path(), "shop", vec![schema()], Format::Jsonl)
                .unwrap();
        sink.write(
            "records",
            &vec![
                Value::Text("a\n\"b".into()),
                Value::Integer(9_007_199_254_740_993),
                Value::Null,
            ],
        )
        .unwrap();
        sink.finish().unwrap();
        assert_eq!(
            std::fs::read_to_string(directory.path().join("shop_records.jsonl")).unwrap(),
            "{\"id\":\"a\\n\\\"b\",\"sku\":9007199254740993,\"note\":null}\n"
        );
    }

    #[test]
    fn csv_streams_composite_keys_in_schema_order_and_omits_zero_row_files() {
        let directory = tempfile::tempdir().unwrap();
        let mut empty = schema();
        empty.name = "empty";
        let mut sink = OutputSink::new(directory.path(), "shop", vec![schema(), empty]).unwrap();
        sink.write(
            "records",
            &vec![Value::Text("a".into()), Value::Integer(1), Value::Null],
        )
        .unwrap();
        sink.write(
            "records",
            &vec![
                Value::Text("a".into()),
                Value::Integer(2),
                Value::Text("quoted,\"text\"\nnext".into()),
            ],
        )
        .unwrap();
        let counts = sink.finish().unwrap();
        assert_eq!(counts["records"], 2);
        assert_eq!(counts["empty"], 0);
        assert!(!directory.path().join("shop_empty.csv").exists());
        assert_eq!(
            std::fs::read_to_string(directory.path().join("shop_records.csv")).unwrap(),
            "id,sku,note\na,1,\na,2,\"quoted,\"\"text\"\"\nnext\"\n"
        );
    }

    #[test]
    fn rejects_invalid_rows_without_reserving_their_keys() {
        let directory = tempfile::tempdir().unwrap();
        let mut sink = OutputSink::new(directory.path(), "shop", vec![schema()]).unwrap();
        let invalid = [
            vec![Value::Text("a".into())],
            vec![Value::Text(String::new()), Value::Integer(1), Value::Null],
            vec![Value::Null, Value::Integer(1), Value::Null],
            vec![
                Value::Text("a".into()),
                Value::Text("1".into()),
                Value::Null,
            ],
            vec![
                Value::Text("a".into()),
                Value::Integer(1),
                Value::Integer(3),
            ],
        ];
        for row in invalid {
            assert!(sink.write("records", &row).is_err());
        }
        let row = vec![Value::Text("a".into()), Value::Integer(1), Value::Null];
        sink.write("records", &row).unwrap();
        assert!(sink.write("records", &row).is_err());
        assert!(sink.write("unknown", &row).is_err());
        assert_eq!(sink.finish().unwrap()["records"], 1);
    }

    #[test]
    fn nullable_key_columns_still_require_values() {
        let directory = tempfile::tempdir().unwrap();
        let mut schema = schema();
        schema.columns[0].nullable = true;
        let mut sink = OutputSink::new(directory.path(), "shop", vec![schema]).unwrap();
        for id in [Value::Null, Value::Text(String::new())] {
            assert!(
                sink.write("records", &vec![id, Value::Integer(1), Value::Null])
                    .is_err()
            );
        }
        assert_eq!(sink.finish().unwrap()["records"], 0);
        assert!(!directory.path().join("shop_records.csv").exists());
    }

    #[test]
    fn serializes_types_without_losing_cents_or_changing_uuid_bytes() {
        let directory = tempfile::tempdir().unwrap();
        let types = [
            ColumnType::Uuid,
            ColumnType::Cents,
            ColumnType::Float,
            ColumnType::Boolean,
            ColumnType::Date,
            ColumnType::Timestamp,
        ];
        let names = ["id", "cost", "rate", "enabled", "date", "time"];
        let schema = EntitySchema {
            name: "values",
            columns: names
                .into_iter()
                .zip(types)
                .map(|(name, column_type)| Column {
                    name,
                    column_type,
                    nullable: false,
                })
                .collect(),
            primary_key: vec!["id"],
        };
        let mut sink = OutputSink::new(directory.path(), "shop", vec![schema]).unwrap();
        sink.write(
            "values",
            &vec![
                Value::Uuid([0xab, 0, 0, 0, 0, 0, 0x40, 0, 0x80, 0, 0, 0, 0, 0, 0, 1]),
                Value::Cents(-123),
                Value::Float(0.25),
                Value::Boolean(true),
                Value::Date(-1),
                Value::Timestamp(1_234_567),
            ],
        )
        .unwrap();
        sink.finish().unwrap();
        assert_eq!(
            std::fs::read_to_string(directory.path().join("shop_values.csv")).unwrap(),
            "id,cost,rate,enabled,date,time\nab000000-0000-4000-8000-000000000001,-123,0.25,True,1969-12-31,1970-01-01T00:00:01\n"
        );
    }
}
