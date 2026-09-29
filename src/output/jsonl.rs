use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use anyhow::{Context, Result};
use flate2::{Compression, GzBuilder, write::GzEncoder};
use serde::ser::{SerializeMap, Serializer};

use super::{EntitySchema, EntityWriter, Row, Value, csv};

pub(super) struct JsonlWriter {
    writer: JsonlOutput,
    columns: Vec<&'static str>,
}

impl JsonlWriter {
    pub(super) fn new(path: &Path, schema: &EntitySchema, compress: bool) -> Result<Self> {
        let file = BufWriter::new(
            File::create(path).with_context(|| format!("creating {}", path.display()))?,
        );
        Ok(Self {
            writer: if compress {
                JsonlOutput::Gzip(
                    GzBuilder::new()
                        .mtime(0)
                        .write(file, Compression::default()),
                )
            } else {
                JsonlOutput::Plain(file)
            },
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
                    object.serialize_entry(name, value)?;
                }
                Value::Float(value) => object.serialize_entry(name, value)?,
                Value::Boolean(value) => object.serialize_entry(name, value)?,
                _ => object.serialize_entry(name, &csv::serialize(value)?)?,
            }
        }
        object.end().context("serializing JSONL row")?;
        self.writer.write_all(b"\n").context("writing JSONL row")
    }

    fn finish(self: Box<Self>) -> Result<()> {
        let mut file = match self.writer {
            JsonlOutput::Plain(file) => file,
            JsonlOutput::Gzip(writer) => writer.finish().context("finishing gzip output")?,
        };
        file.flush().context("flushing JSONL output")
    }
}

enum JsonlOutput {
    Plain(BufWriter<File>),
    Gzip(GzEncoder<BufWriter<File>>),
}

impl Write for JsonlOutput {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Plain(writer) => writer.write(bytes),
            Self::Gzip(writer) => writer.write(bytes),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Plain(writer) => writer.flush(),
            Self::Gzip(writer) => writer.flush(),
        }
    }
}
