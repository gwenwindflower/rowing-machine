use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn worker_counts_preserve_every_file_across_themes_formats_and_compression() {
    for theme in ["plain", "fantasy_rpg"] {
        for (format, compressed) in [
            ("csv", false),
            ("jsonl", false),
            ("jsonl", true),
            ("parquet", false),
            ("parquet", true),
        ] {
            let directory = tempfile::tempdir().unwrap();
            for workers in ["1", "8"] {
                let mut command = cargo_bin_cmd!("rowing-machine");
                command.args([
                    "--years",
                    "1",
                    "--scale",
                    "1",
                    "--seed",
                    "42",
                    "--quiet",
                    "--theme",
                    theme,
                    "--format",
                    format,
                    "--workers",
                    workers,
                ]);
                if compressed {
                    command.arg("--compress");
                }
                command
                    .arg("--output-dir")
                    .arg(directory.path().join(workers))
                    .assert()
                    .success()
                    .stdout("")
                    .stderr("");
            }
            assert_eq!(
                std::fs::read_dir(directory.path().join("1"))
                    .unwrap()
                    .count(),
                7
            );
            assert_eq!(
                std::fs::read_dir(directory.path().join("8"))
                    .unwrap()
                    .count(),
                7
            );
            for entry in std::fs::read_dir(directory.path().join("1")).unwrap() {
                let entry = entry.unwrap();
                assert_eq!(
                    std::fs::read(entry.path()).unwrap(),
                    std::fs::read(directory.path().join("8").join(entry.file_name())).unwrap(),
                    "{theme} {format} compressed={compressed}: {:?}",
                    entry.file_name()
                );
            }
        }
    }
}
