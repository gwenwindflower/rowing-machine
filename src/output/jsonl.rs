use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use anyhow::{Context, Result};
use flate2::{Compression, GzBuilder, write::GzEncoder};

use super::{EntitySchema, EntityWriter, Row, Value, csv};

pub(super) struct JsonlWriter {
    writer: JsonlOutput,
    names: Vec<Vec<u8>>,
    buffer: Vec<u8>,
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
            names: encoded_names(schema)?,
            buffer: Vec::new(),
        })
    }
}

impl EntityWriter for JsonlWriter {
    fn write_row(&mut self, row: &Row) -> Result<()> {
        self.buffer.clear();
        append_row(&mut self.buffer, &self.names, row)?;
        self.writer
            .write_all(&self.buffer)
            .context("writing JSONL row")
    }

    fn write_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        self.writer.write_all(bytes).context("writing JSONL rows")
    }

    fn finish(self: Box<Self>) -> Result<()> {
        let mut file = match self.writer {
            JsonlOutput::Plain(file) => file,
            JsonlOutput::Gzip(writer) => writer.finish().context("finishing gzip output")?,
        };
        file.flush().context("flushing JSONL output")
    }
}

fn encoded_names(schema: &EntitySchema) -> Result<Vec<Vec<u8>>> {
    schema
        .columns
        .iter()
        .map(|column| serde_json::to_vec(column.name).context("encoding JSONL column name"))
        .collect()
}

pub(super) fn encode(schema: &EntitySchema, rows: &[&Row]) -> Result<Vec<u8>> {
    let names = encoded_names(schema)?;
    let mut bytes = Vec::new();
    for row in rows {
        append_row(&mut bytes, &names, row)?;
    }
    Ok(bytes)
}

fn append_row(bytes: &mut Vec<u8>, names: &[Vec<u8>], row: &Row) -> Result<()> {
    bytes.push(b'{');
    for (index, (name, value)) in names.iter().zip(row).enumerate() {
        if index > 0 {
            bytes.push(b',');
        }
        bytes.extend_from_slice(name);
        bytes.push(b':');
        match value {
            Value::Null => bytes.extend_from_slice(b"null"),
            Value::Integer(value) | Value::Cents(value) => csv::append_integer(bytes, *value),
            Value::Float(value) => serde_json::to_writer(&mut *bytes, value)?,
            Value::Boolean(value) => {
                bytes.extend_from_slice(if *value { b"true" } else { b"false" });
            }
            Value::Text(value) => serde_json::to_writer(&mut *bytes, value)?,
            _ => {
                bytes.push(b'"');
                csv::append_value(bytes, value)?;
                bytes.push(b'"');
            }
        }
    }
    bytes.extend_from_slice(b"}\n");
    Ok(())
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
