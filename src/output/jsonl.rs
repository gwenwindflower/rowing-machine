use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use anyhow::{Context, Result};
use serde::ser::{SerializeMap, Serializer};

use super::{EntitySchema, EntityWriter, Row, Value, csv};

pub(super) struct JsonlWriter {
    writer: BufWriter<File>,
    columns: Vec<&'static str>,
}

impl JsonlWriter {
    pub(super) fn new(path: &Path, schema: &EntitySchema) -> Result<Self> {
        Ok(Self {
            writer: BufWriter::new(
                File::create(path).with_context(|| format!("creating {}", path.display()))?,
            ),
            columns: schema.columns.iter().map(|column| column.name).collect(),
        })
    }
}

impl EntityWriter for JsonlWriter {
    fn write_row(&mut self, row: &Row) -> Result<()> {
        let mut serializer = serde_json::Serializer::new(&mut self.writer);
        let mut object = serializer.serialize_map(Some(row.len()))?;
        for (name, value) in self.columns.iter().zip(row) {
            match value {
                Value::Null => object.serialize_entry(name, &Option::<()>::None)?,
                Value::Integer(value) | Value::Cents(value) => {
                    object.serialize_entry(name, value)?
                }
                Value::Float(value) => object.serialize_entry(name, value)?,
                Value::Boolean(value) => object.serialize_entry(name, value)?,
                _ => object.serialize_entry(name, &csv::serialize(value)?)?,
            }
        }
        object.end().context("serializing JSONL row")?;
        self.writer.write_all(b"\n").context("writing JSONL row")
    }

    fn finish(mut self: Box<Self>) -> Result<()> {
        self.writer.flush().context("flushing JSONL output")
    }
}
