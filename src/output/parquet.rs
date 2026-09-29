use std::fs::File;
use std::path::Path;
use std::sync::Arc;

use anyhow::{Context, Result, bail};
use arrow::array::{
    ArrayRef, BooleanArray, Date32Array, Float64Array, Int64Array, StringArray,
    TimestampMicrosecondArray,
};
use arrow::datatypes::{DataType, Field, Schema, SchemaRef, TimeUnit};
use arrow::record_batch::RecordBatch;
use parquet::arrow::ArrowWriter;
use parquet::basic::{Compression, ZstdLevel};
use parquet::file::properties::WriterProperties;

use super::{ColumnType, EntitySchema, EntityWriter, Row, Value, csv};

const TARGET_ROW_GROUPS: usize = 8;

pub(super) struct ParquetWriter {
    writer: ArrowWriter<File>,
    schema: SchemaRef,
    column_types: Vec<ColumnType>,
    rows: Vec<Row>,
    row_group_size: usize,
}

impl ParquetWriter {
    pub(super) fn new(
        path: &Path,
        schema: &EntitySchema,
        estimated_rows: usize,
        compress: bool,
    ) -> Result<Self> {
        let row_group_size = estimated_rows
            .div_ceil(TARGET_ROW_GROUPS)
            .clamp(1_024, 65_536);
        let arrow_schema = Arc::new(Schema::new(
            schema
                .columns
                .iter()
                .map(|column| {
                    let data_type = match column.column_type {
                        ColumnType::Uuid | ColumnType::Text => DataType::Utf8,
                        ColumnType::Integer | ColumnType::Cents => DataType::Int64,
                        ColumnType::Float => DataType::Float64,
                        ColumnType::Boolean => DataType::Boolean,
                        ColumnType::Date => DataType::Date32,
                        ColumnType::Timestamp => {
                            DataType::Timestamp(TimeUnit::Microsecond, Some("UTC".into()))
                        }
                    };
                    Field::new(column.name, data_type, column.nullable)
                })
                .collect::<Vec<_>>(),
        ));
        let compression = if compress {
            Compression::ZSTD(ZstdLevel::default())
        } else {
            Compression::UNCOMPRESSED
        };
        let properties = WriterProperties::builder()
            .set_max_row_group_row_count(Some(row_group_size))
            .set_compression(compression)
            .build();
        let file = File::create(path).with_context(|| format!("creating {}", path.display()))?;
        let writer = ArrowWriter::try_new(file, Arc::clone(&arrow_schema), Some(properties))
            .with_context(|| format!("opening Parquet writer for {}", path.display()))?;
        Ok(Self {
            writer,
            schema: arrow_schema,
            column_types: schema
                .columns
                .iter()
                .map(|column| column.column_type)
                .collect(),
            rows: Vec::new(),
            row_group_size,
        })
    }

    fn flush_rows(&mut self) -> Result<()> {
        if self.rows.is_empty() {
            return Ok(());
        }
        let arrays = self
            .column_types
            .iter()
            .enumerate()
            .map(|(index, column_type)| column_array(&self.rows, index, *column_type))
            .collect::<Result<Vec<_>>>()?;
        let batch = RecordBatch::try_new(Arc::clone(&self.schema), arrays)
            .context("building Parquet row group")?;
        self.writer
            .write(&batch)
            .context("writing Parquet row group")?;
        self.rows.clear();
        Ok(())
    }
}

impl EntityWriter for ParquetWriter {
    fn write_row(&mut self, row: &Row) -> Result<()> {
        self.rows.push(row.clone());
        if self.rows.len() == self.row_group_size {
            self.flush_rows()?;
        }
        Ok(())
    }

    fn finish(mut self: Box<Self>) -> Result<()> {
        self.flush_rows()?;
        self.writer.close().context("closing Parquet output")?;
        Ok(())
    }
}

fn column_array(rows: &[Row], index: usize, column_type: ColumnType) -> Result<ArrayRef> {
    macro_rules! primitive_values {
        ($($variant:ident)|+) => {
            rows.iter().map(|row| match &row[index] {
                $(Value::$variant(value))|+ => Ok(Some(*value)),
                Value::Null => Ok(None),
                _ => bail!("incorrect value type in Parquet column {index}"),
            }).collect::<Result<Vec<_>>>()?
        };
    }
    Ok(match column_type {
        ColumnType::Uuid | ColumnType::Text => {
            let values = rows
                .iter()
                .map(|row| match &row[index] {
                    Value::Null => Ok(None),
                    value => csv::serialize(value).map(Some),
                })
                .collect::<Result<Vec<_>>>()?;
            Arc::new(StringArray::from(values))
        }
        ColumnType::Integer | ColumnType::Cents => {
            Arc::new(Int64Array::from(primitive_values!(Integer | Cents)))
        }
        ColumnType::Float => Arc::new(Float64Array::from(primitive_values!(Float))),
        ColumnType::Boolean => Arc::new(BooleanArray::from(primitive_values!(Boolean))),
        ColumnType::Date => Arc::new(Date32Array::from(primitive_values!(Date))),
        ColumnType::Timestamp => Arc::new(
            TimestampMicrosecondArray::from(primitive_values!(Timestamp)).with_timezone("UTC"),
        ),
    })
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use arrow::array::{
        Array, BooleanArray, Date32Array, Float64Array, Int64Array, StringArray,
        TimestampMicrosecondArray,
    };
    use arrow::datatypes::{DataType, TimeUnit};
    use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
    use parquet::basic::{Compression, ZstdLevel};

    use super::*;
    use crate::output::{Column, ColumnType, Value};

    fn schema() -> EntitySchema {
        EntitySchema {
            name: "records",
            columns: [
                ("id", ColumnType::Uuid),
                ("text", ColumnType::Text),
                ("integer", ColumnType::Integer),
                ("cents", ColumnType::Cents),
                ("float", ColumnType::Float),
                ("boolean", ColumnType::Boolean),
                ("date", ColumnType::Date),
                ("timestamp", ColumnType::Timestamp),
            ]
            .into_iter()
            .map(|(name, column_type)| Column {
                name,
                column_type,
                nullable: true,
            })
            .collect(),
            primary_key: vec!["id"],
        }
    }

    fn row() -> Row {
        vec![
            Value::Uuid([0xab, 0, 0, 0, 0, 0, 0x40, 0, 0x80, 0, 0, 0, 0, 0, 0, 1]),
            Value::Text("quoted,\"text\"\nnext".into()),
            Value::Integer(-42),
            Value::Cents(9_007_199_254_740_993),
            Value::Float(0.125),
            Value::Boolean(true),
            Value::Date(-1),
            Value::Timestamp(1_234_567),
        ]
    }

    #[test]
    fn parquet_preserves_order_types_values_and_nulls_with_identical_bytes() {
        let directory = tempfile::tempdir().unwrap();
        for compress in [false, true] {
            let paths = [
                directory.path().join("first.parquet"),
                directory.path().join("second.parquet"),
            ];
            for path in &paths {
                let mut writer = ParquetWriter::new(path, &schema(), 2, compress).unwrap();
                writer.write_row(&row()).unwrap();
                writer.write_row(&vec![Value::Null; 8]).unwrap();
                Box::new(writer).finish().unwrap();
            }
            assert_eq!(
                std::fs::read(&paths[0]).unwrap(),
                std::fs::read(&paths[1]).unwrap()
            );
            let builder =
                ParquetRecordBatchReaderBuilder::try_new(File::open(&paths[0]).unwrap()).unwrap();
            let actual_schema = builder.schema();
            for (field, column) in actual_schema.fields().iter().zip(schema().columns) {
                assert_eq!(field.name(), column.name);
                assert!(field.is_nullable());
            }
            assert_eq!(actual_schema.field(3).data_type(), &DataType::Int64);
            assert_eq!(
                actual_schema.field(7).data_type(),
                &DataType::Timestamp(TimeUnit::Microsecond, Some("UTC".into()))
            );
            let batch = builder
                .with_batch_size(2)
                .build()
                .unwrap()
                .next()
                .unwrap()
                .unwrap();
            assert_eq!(batch.num_rows(), 2);
            for column in batch.columns() {
                assert!(!column.is_null(0));
                assert!(column.is_null(1));
            }
            assert_eq!(
                batch
                    .column(0)
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .unwrap()
                    .value(0),
                "ab000000-0000-4000-8000-000000000001"
            );
            assert_eq!(
                batch
                    .column(1)
                    .as_any()
                    .downcast_ref::<StringArray>()
                    .unwrap()
                    .value(0),
                "quoted,\"text\"\nnext"
            );
            assert_eq!(
                batch
                    .column(2)
                    .as_any()
                    .downcast_ref::<Int64Array>()
                    .unwrap()
                    .value(0),
                -42
            );
            assert_eq!(
                batch
                    .column(3)
                    .as_any()
                    .downcast_ref::<Int64Array>()
                    .unwrap()
                    .value(0),
                9_007_199_254_740_993
            );
            assert_eq!(
                batch
                    .column(4)
                    .as_any()
                    .downcast_ref::<Float64Array>()
                    .unwrap()
                    .value(0),
                0.125
            );
            assert!(
                batch
                    .column(5)
                    .as_any()
                    .downcast_ref::<BooleanArray>()
                    .unwrap()
                    .value(0)
            );
            assert_eq!(
                batch
                    .column(6)
                    .as_any()
                    .downcast_ref::<Date32Array>()
                    .unwrap()
                    .value(0),
                -1
            );
            assert_eq!(
                batch
                    .column(7)
                    .as_any()
                    .downcast_ref::<TimestampMicrosecondArray>()
                    .unwrap()
                    .value(0),
                1_234_567
            );
        }
    }

    #[test]
    fn parquet_bounds_row_groups_and_selects_column_compression() {
        let directory = tempfile::tempdir().unwrap();
        for (estimated_rows, expected_groups, compress) in [
            (0, vec![1_024, 1_024, 2], false),
            (8_192, vec![1_024, 1_024, 2], true),
            (16_384, vec![2_048, 2], false),
        ] {
            let path = directory.path().join("groups.parquet");
            let mut writer =
                ParquetWriter::new(&path, &schema(), estimated_rows, compress).unwrap();
            for _ in 0..2_050 {
                writer.write_row(&row()).unwrap();
            }
            Box::new(writer).finish().unwrap();
            let builder =
                ParquetRecordBatchReaderBuilder::try_new(File::open(path).unwrap()).unwrap();
            let groups = builder.metadata().row_groups();
            assert_eq!(
                groups
                    .iter()
                    .map(parquet::file::metadata::RowGroupMetaData::num_rows)
                    .collect::<Vec<_>>(),
                expected_groups
            );
            let compression = if compress {
                Compression::ZSTD(ZstdLevel::default())
            } else {
                Compression::UNCOMPRESSED
            };
            for group in groups {
                assert!(
                    group
                        .columns()
                        .iter()
                        .all(|column| column.compression() == compression)
                );
            }
        }
    }
}
