//! Scenario-agnostic entity schemas and rows handed to format writers.

use std::collections::{BTreeMap, BTreeSet};
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
    keys: BTreeSet<Vec<String>>,
    writer: Option<Box<dyn EntityWriter>>,
    count: u64,
    estimated_rows: usize,
}

pub struct OutputSink {
    directory: PathBuf,
    prefix: String,
    format: Format,
    entities: BTreeMap<&'static str, EntityOutput>,
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
                    keys: BTreeSet::new(),
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
            .get_mut(entity)
            .with_context(|| format!("unknown output entity {entity}"))?;
        ensure!(
            row.len() == output.schema.columns.len(),
            "entity {entity}: expected {} fields, got {}",
            output.schema.columns.len(),
            row.len()
        );
        for (index, (column, value)) in output.schema.columns.iter().zip(row).enumerate() {
            let empty = matches!(value, Value::Null)
                || matches!(value, Value::Text(text) if text.is_empty());
            ensure!(
                !empty || (column.nullable && !output.key_indices.contains(&index)),
                "entity {entity}: column {} must not be empty",
                column.name
            );
            ensure!(
                matches!(value, Value::Null) || value.column_type() == column.column_type,
                "entity {entity}: column {} has incorrect value type",
                column.name
            );
        }
        let fields = row
            .iter()
            .map(csv::serialize)
            .collect::<Result<Vec<_>>>()
            .with_context(|| format!("serializing entity {entity}"))?;
        let key = output
            .key_indices
            .iter()
            .map(|&index| fields[index].clone())
            .collect::<Vec<_>>();
        ensure!(
            !output.keys.contains(&key),
            "entity {entity}: duplicate primary key {key:?}"
        );
        if output.writer.is_none() {
            std::fs::create_dir_all(&self.directory).with_context(|| {
                format!("creating output directory {}", self.directory.display())
            })?;
            let path = self.directory.join(format!(
                "{}_{entity}.{}",
                self.prefix,
                self.format.extension()
            ));
            output.writer = Some(match self.format {
                Format::Csv => Box::new(csv::CsvWriter::new(&path, &output.schema)?),
                Format::Jsonl => Box::new(jsonl::JsonlWriter::new(&path, &output.schema)?),
                Format::Parquet => Box::new(parquet::ParquetWriter::new(
                    &path,
                    &output.schema,
                    output.estimated_rows,
                    false,
                )?),
            });
        }
        output
            .writer
            .as_mut()
            .context("entity writer was not opened")?
            .write_row(row)?;
        output.keys.insert(key);
        output.count += 1;
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
pub trait EntityWriter {
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
