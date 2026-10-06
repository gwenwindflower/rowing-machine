use assert_cmd::cargo::cargo_bin_cmd;
use std::{fs::File, io::Read};

#[test]
fn gzip_jsonl_decompresses_to_the_plain_jsonl_bytes() {
    let plain = tempfile::tempdir().unwrap();
    let compressed = tempfile::tempdir().unwrap();
    for (directory, compress) in [(&plain, false), (&compressed, true)] {
        let mut command = cargo_bin_cmd!("rowing-machine");
        command
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
            .arg(directory.path());
        if compress {
            command.arg("--compress");
        }
        command.assert().success().stdout("").stderr("");
    }
    assert_eq!(std::fs::read_dir(compressed.path()).unwrap().count(), 7);
    for entry in std::fs::read_dir(compressed.path()).unwrap() {
        let entry = entry.unwrap();
        let mut decoded = Vec::new();
        flate2::read::GzDecoder::new(File::open(entry.path()).unwrap())
            .read_to_end(&mut decoded)
            .unwrap();
        let name = entry.file_name();
        let name = name.to_str().unwrap().strip_suffix(".gz").unwrap();
        assert_eq!(decoded, std::fs::read(plain.path().join(name)).unwrap());
    }
}

#[test]
fn compressed_parquet_uses_zstd_on_every_column() {
    use parquet::{arrow::arrow_reader::ParquetRecordBatchReaderBuilder, basic::Compression};
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
            "--compress",
            "--output-dir",
        ])
        .arg(directory.path())
        .assert()
        .success();
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 7);
    for entry in std::fs::read_dir(directory.path()).unwrap() {
        let entry = entry.unwrap();
        assert_eq!(entry.path().extension().unwrap(), "parquet");
        let builder =
            ParquetRecordBatchReaderBuilder::try_new(File::open(entry.path()).unwrap()).unwrap();
        for group in builder.metadata().row_groups() {
            assert!(
                group
                    .columns()
                    .iter()
                    .all(|column| matches!(column.compression(), Compression::ZSTD(_)))
            );
        }
        assert!(builder.build().unwrap().all(|batch| batch.is_ok()));
    }
}

#[test]
fn csv_compression_is_rejected_before_output_is_created() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("output");
    let result = cargo_bin_cmd!("rowing-machine")
        .args(["--compress", "--output-dir"])
        .arg(&path)
        .assert()
        .failure();
    let error = String::from_utf8_lossy(&result.get_output().stderr);
    for word in ["--compress", "csv", "jsonl", "parquet"] {
        assert!(error.contains(word), "{error}");
    }
    assert!(!path.exists());
}
