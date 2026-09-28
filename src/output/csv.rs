use std::fmt::Write;
use std::fs::File;
use std::path::Path;

use anyhow::{Context, Result, ensure};

use super::{EntitySchema, EntityWriter, Row, Value};

pub(super) struct CsvWriter {
    writer: ::csv::Writer<File>,
}

impl CsvWriter {
    pub(super) fn new(path: &Path, schema: &EntitySchema) -> Result<Self> {
        let file = File::create(path).with_context(|| format!("creating {}", path.display()))?;
        let mut writer = ::csv::Writer::from_writer(file);
        writer
            .write_record(schema.columns.iter().map(|column| column.name))
            .with_context(|| format!("writing header to {}", path.display()))?;
        Ok(Self { writer })
    }

    pub(super) fn write_fields(&mut self, fields: &[String]) -> Result<()> {
        self.writer.write_record(fields).context("writing CSV row")
    }
}

impl EntityWriter for CsvWriter {
    fn write_row(&mut self, row: &Row) -> Result<()> {
        self.write_fields(&row.iter().map(serialize).collect::<Result<Vec<_>>>()?)
    }

    fn finish(mut self: Box<Self>) -> Result<()> {
        self.writer.flush().context("flushing CSV output")
    }
}

pub(super) fn serialize(value: &Value) -> Result<String> {
    Ok(match value {
        Value::Null => String::new(),
        Value::Text(value) => value.clone(),
        Value::Uuid(bytes) => {
            ensure!(
                bytes[6] >> 4 == 4 && bytes[8] >> 6 == 2,
                "UUID must have version 4 and RFC 4122 variant bits"
            );
            let mut hex = String::with_capacity(32);
            for byte in bytes {
                write!(hex, "{byte:02x}")?;
            }
            format!(
                "{}-{}-{}-{}-{}",
                &hex[..8],
                &hex[8..12],
                &hex[12..16],
                &hex[16..20],
                &hex[20..]
            )
        }
        Value::Integer(value) | Value::Cents(value) => value.to_string(),
        Value::Float(value) => {
            ensure!(value.is_finite(), "float must be finite");
            value.to_string()
        }
        Value::Boolean(value) => if *value { "True" } else { "False" }.to_owned(),
        Value::Date(days) => jiff::Timestamp::from_second(i64::from(*days) * 86_400)?
            .strftime("%Y-%m-%d")
            .to_string(),
        Value::Timestamp(micros) => jiff::Timestamp::from_microsecond(*micros)?
            .strftime("%Y-%m-%dT%H:%M:%S")
            .to_string(),
    })
}
