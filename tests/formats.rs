use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn parquet_orders_read_back_with_typed_cents_and_utc_timestamps() {
    use arrow::datatypes::{DataType, TimeUnit};
    use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
    let directory = tempfile::tempdir().unwrap();
    cargo_bin_cmd!("rowing-machine")
        .args([
            "--years",
            "1",
            "--scale",
            "1",
            "--seed",
            "42",
            "--quiet",
            "--format",
            "parquet",
            "--output-dir",
        ])
        .arg(directory.path())
        .assert()
        .success();
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 7);
    let reader = ParquetRecordBatchReaderBuilder::try_new(
        std::fs::File::open(directory.path().join("raw_orders.parquet")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        reader
            .schema()
            .field_with_name("order_total")
            .unwrap()
            .data_type(),
        &DataType::Int64
    );
    assert_eq!(
        reader
            .schema()
            .field_with_name("ordered_at")
            .unwrap()
            .data_type(),
        &DataType::Timestamp(TimeUnit::Microsecond, Some("UTC".into()))
    );
    assert!(reader.build().unwrap().next().unwrap().unwrap().num_rows() > 0);
}

#[test]
fn jsonl_preserves_native_values_and_column_order() {
    let directory = tempfile::tempdir().unwrap();
    cargo_bin_cmd!("rowing-machine")
        .args([
            "--years",
            "1",
            "--scale",
            "1",
            "--seed",
            "42",
            "--quiet",
            "--format",
            "jsonl",
            "--output-dir",
        ])
        .arg(directory.path())
        .assert()
        .success()
        .stdout("")
        .stderr("");
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 7);
    let orders = std::fs::read_to_string(directory.path().join("raw_orders.jsonl")).unwrap();
    let line = orders.lines().next().unwrap();
    assert!(line.starts_with("{\"id\":"));
    let row: serde_json::Value = serde_json::from_str(line).unwrap();
    assert!(row["order_total"].is_i64());
    assert!(row["ordered_at"].as_str().unwrap().contains('T'));
    let supplies = std::fs::read_to_string(directory.path().join("raw_supplies.jsonl")).unwrap();
    let row: serde_json::Value = serde_json::from_str(supplies.lines().next().unwrap()).unwrap();
    assert!(row["volatile"].is_boolean());
}
