use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn worker_counts_preserve_every_file_across_themes_formats_and_compression() {
    let available = std::thread::available_parallelism()
        .unwrap()
        .get()
        .to_string();
    for (scenario, theme) in [
        ("ecommerce", "plain"),
        ("ecommerce", "fantasy_rpg"),
        ("saas", "plain"),
    ] {
        for (format, compressed) in [
            ("csv", false),
            ("jsonl", false),
            ("jsonl", true),
            ("parquet", false),
            ("parquet", true),
        ] {
            let directory = tempfile::tempdir().unwrap();
            for (label, workers) in [("serial", "1"), ("parallel", available.as_str())] {
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
                    "--scenario",
                    scenario,
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
                    .arg(directory.path().join(label))
                    .assert()
                    .success()
                    .stdout("")
                    .stderr("");
            }
            let entity_count = std::fs::read_dir(directory.path().join("serial"))
                .unwrap()
                .count();
            assert!(entity_count > 0);
            assert_eq!(
                std::fs::read_dir(directory.path().join("parallel"))
                    .unwrap()
                    .count(),
                entity_count
            );
            for entry in std::fs::read_dir(directory.path().join("serial")).unwrap() {
                let entry = entry.unwrap();
                assert_eq!(
                    std::fs::read(entry.path()).unwrap(),
                    std::fs::read(directory.path().join("parallel").join(entry.file_name()))
                        .unwrap(),
                    "{scenario} {theme} {format} compressed={compressed}: {:?}",
                    entry.file_name()
                );
            }
        }
    }
}
