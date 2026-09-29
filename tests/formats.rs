use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn parquet_runs_byte_match_and_read_back_typed_orders() {
    use arrow::datatypes::{DataType, TimeUnit};
    use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    for directory in [&first, &second] {
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
    }
    assert_eq!(std::fs::read_dir(first.path()).unwrap().count(), 7);
    for entry in std::fs::read_dir(first.path()).unwrap() {
        let entry = entry.unwrap();
        assert_eq!(
            std::fs::read(entry.path()).unwrap(),
            std::fs::read(second.path().join(entry.file_name())).unwrap()
        );
    }
    let reader = ParquetRecordBatchReaderBuilder::try_new(
        std::fs::File::open(first.path().join("raw_orders.parquet")).unwrap(),
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
fn jsonl_runs_byte_match_and_preserve_native_values_and_column_order() {
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    for directory in [&first, &second] {
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
    }
    assert_eq!(std::fs::read_dir(first.path()).unwrap().count(), 7);
    for entry in std::fs::read_dir(first.path()).unwrap() {
        let entry = entry.unwrap();
        assert_eq!(
            std::fs::read(entry.path()).unwrap(),
            std::fs::read(second.path().join(entry.file_name())).unwrap()
        );
    }
    let orders = std::fs::read_to_string(first.path().join("raw_orders.jsonl")).unwrap();
    let line = orders.lines().next().unwrap();
    assert!(line.starts_with("{\"id\":"));
    let row: serde_json::Value = serde_json::from_str(line).unwrap();
    assert!(row["order_total"].is_i64());
    assert!(row["ordered_at"].as_str().unwrap().contains('T'));
    let supplies = std::fs::read_to_string(first.path().join("raw_supplies.jsonl")).unwrap();
    let row: serde_json::Value = serde_json::from_str(supplies.lines().next().unwrap()).unwrap();
    assert!(row["volatile"].is_boolean());
}
