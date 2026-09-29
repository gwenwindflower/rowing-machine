use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use anyhow::{Context, Result, ensure};

use super::{EntitySchema, EntityWriter, Row, Value};

pub(super) struct CsvWriter {
    writer: BufWriter<File>,
    buffer: Vec<u8>,
}

impl CsvWriter {
    pub(super) fn new(path: &Path, schema: &EntitySchema) -> Result<Self> {
        let file = File::create(path).with_context(|| format!("creating {}", path.display()))?;
        let mut header = ::csv::Writer::from_writer(Vec::new());
        header
            .write_record(schema.columns.iter().map(|column| column.name))
            .with_context(|| format!("writing header to {}", path.display()))?;
        let mut writer = BufWriter::new(file);
        writer.write_all(&header.into_inner()?)?;
        Ok(Self {
            writer,
            buffer: Vec::new(),
        })
    }
}

impl EntityWriter for CsvWriter {
    fn write_row(&mut self, row: &Row) -> Result<()> {
        self.buffer.clear();
        append_row(&mut self.buffer, row)?;
        self.writer
            .write_all(&self.buffer)
            .context("writing CSV row")
    }

    fn write_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        self.writer.write_all(bytes).context("writing CSV rows")
    }

    fn finish(mut self: Box<Self>) -> Result<()> {
        self.writer.flush().context("flushing CSV output")
    }
}

pub(super) fn serialize(value: &Value) -> Result<String> {
    let mut bytes = Vec::new();
    append_value(&mut bytes, value)?;
    String::from_utf8(bytes).context("serializing UTF-8 value")
}

pub(super) fn encode(_schema: &EntitySchema, rows: &[&Row]) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    for row in rows {
        append_row(&mut bytes, row)?;
    }
    Ok(bytes)
}

fn append_row(bytes: &mut Vec<u8>, row: &Row) -> Result<()> {
    for (index, value) in row.iter().enumerate() {
        if index > 0 {
            bytes.push(b',');
        }
        let Value::Text(text) = value else {
            if row.len() == 1 && matches!(value, Value::Null) {
                bytes.extend_from_slice(b"\"\"");
            } else {
                append_value(bytes, value)?;
            }
            continue;
        };
        let field = text.as_bytes();
        if field
            .iter()
            .any(|byte| matches!(byte, b',' | b'"' | b'\r' | b'\n'))
            || (row.len() == 1 && field.is_empty())
        {
            bytes.push(b'"');
            for byte in field {
                bytes.push(*byte);
                if *byte == b'"' {
                    bytes.push(b'"');
                }
            }
            bytes.push(b'"');
        } else {
            bytes.extend_from_slice(field);
        }
    }
    bytes.push(b'\n');
    Ok(())
}

pub(super) fn append_integer(bytes: &mut Vec<u8>, value: i64) {
    if value < 0 {
        bytes.push(b'-');
    }
    append_digits(bytes, value.unsigned_abs(), 1);
}

fn append_digits(bytes: &mut Vec<u8>, mut value: u64, width: usize) {
    let mut digits = [b'0'; 20];
    let mut start = digits.len();
    loop {
        start -= 1;
        digits[start] = b'0' + u8::try_from(value % 10).expect("decimal digit");
        value /= 10;
        if value == 0 {
            break;
        }
    }
    bytes.extend_from_slice(&digits[start.min(digits.len() - width)..]);
}

fn append_date(bytes: &mut Vec<u8>, date: jiff::civil::Date) {
    let year = i64::from(date.year());
    if year < 0 {
        bytes.push(b'-');
    }
    append_digits(bytes, year.unsigned_abs(), if year < 0 { 3 } else { 4 });
    bytes.push(b'-');
    append_digits(bytes, u64::from(date.month().unsigned_abs()), 2);
    bytes.push(b'-');
    append_digits(bytes, u64::from(date.day().unsigned_abs()), 2);
}

pub(super) fn append_value(bytes: &mut Vec<u8>, value: &Value) -> Result<()> {
    match value {
        Value::Null => {}
        Value::Text(value) => bytes.extend_from_slice(value.as_bytes()),
        Value::Uuid(uuid) => {
            ensure!(
                uuid[6] >> 4 == 4 && uuid[8] >> 6 == 2,
                "UUID must have version 4 and RFC 4122 variant bits"
            );
            append_uuid(bytes, uuid);
        }
        Value::Integer(value) | Value::Cents(value) => append_integer(bytes, *value),
        Value::Float(value) => {
            ensure!(value.is_finite(), "float must be finite");
            bytes.extend_from_slice(value.to_string().as_bytes());
        }
        Value::Boolean(value) => bytes.extend_from_slice(if *value { b"True" } else { b"False" }),
        Value::Date(days) => append_date(
            bytes,
            jiff::Timestamp::from_second(i64::from(*days) * 86_400)?
                .to_zoned(jiff::tz::TimeZone::UTC)
                .date(),
        ),
        Value::Timestamp(micros) => {
            let time =
                jiff::Timestamp::from_microsecond(*micros)?.to_zoned(jiff::tz::TimeZone::UTC);
            append_date(bytes, time.date());
            bytes.push(b'T');
            append_digits(bytes, u64::from(time.hour().unsigned_abs()), 2);
            bytes.push(b':');
            append_digits(bytes, u64::from(time.minute().unsigned_abs()), 2);
            bytes.push(b':');
            append_digits(bytes, u64::from(time.second().unsigned_abs()), 2);
        }
    }
    Ok(())
}

fn append_uuid(output: &mut Vec<u8>, bytes: &[u8; 16]) {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for (index, byte) in bytes.iter().enumerate() {
        if matches!(index, 4 | 6 | 8 | 10) {
            output.push(b'-');
        }
        output.push(HEX[usize::from(byte >> 4)]);
        output.push(HEX[usize::from(byte & 15)]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batch_encoding_preserves_numeric_limits_dates_and_csv_escaping() {
        let schema = EntitySchema {
            name: "values",
            columns: vec![],
            primary_key: vec![],
        };
        let row = vec![
            Value::Integer(i64::MIN),
            Value::Integer(i64::MAX),
            Value::Timestamp(-1),
            Value::Date(11_016),
            Value::Text("a,\"b\"\r\n雪".into()),
            Value::Null,
        ];
        assert_eq!(
            String::from_utf8(encode(&schema, &[&row]).unwrap()).unwrap(),
            "-9223372036854775808,9223372036854775807,1969-12-31T23:59:59,2000-02-29,\"a,\"\"b\"\"\r\n雪\",\n"
        );
        assert_eq!(encode(&schema, &[&vec![Value::Null]]).unwrap(), b"\"\"\n");
    }

    #[test]
    fn timestamps_preserve_calendar_boundaries_and_discard_subseconds() {
        for text in [
            "0000-01-01T00:00:00Z",
            "-000001-12-31T23:59:59Z",
            "1900-03-01T00:00:00Z",
            "2000-02-29T23:59:59.999999Z",
            "9999-01-01T23:59:59Z",
        ] {
            let timestamp: jiff::Timestamp = text.parse().unwrap();
            assert_eq!(
                serialize(&Value::Timestamp(timestamp.as_microsecond())).unwrap(),
                timestamp.strftime("%Y-%m-%dT%H:%M:%S").to_string()
            );
        }
    }

    #[test]
    fn serializers_reject_invalid_uuids_floats_and_dates() {
        for value in [
            Value::Uuid([0; 16]),
            Value::Float(f64::INFINITY),
            Value::Float(f64::NAN),
            Value::Date(i32::MAX),
            Value::Timestamp(i64::MIN),
        ] {
            assert!(serialize(&value).is_err());
        }
    }

    #[test]
    fn csv_text_encoding_matches_csv_records() {
        let values = ["", "plain", "a,b", "\"quoted\"", "\r", "\n", "雪\0", " a "];
        for first in values {
            for second in values {
                let mut expected = ::csv::Writer::from_writer(Vec::new());
                expected.write_record([first, second]).unwrap();
                let mut actual = Vec::new();
                append_row(
                    &mut actual,
                    &vec![Value::Text(first.into()), Value::Text(second.into())],
                )
                .unwrap();
                assert_eq!(actual, expected.into_inner().unwrap());
            }
        }
    }
}
