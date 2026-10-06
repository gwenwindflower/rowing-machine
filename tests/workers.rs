use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn worker_counts_preserve_every_file_across_themes_formats_and_compression() {
    let available = std::thread::available_parallelism()
        .unwrap()
        .get()
        .to_string();
    for (scenario, theme, scale, entities) in [
        ("ecommerce", "plain", "1", 7),
        ("ecommerce", "fantasy_rpg", "1", 7),
        ("saas", "plain", "4", 16),
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
                    scale,
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
            assert_eq!(entity_count, entities, "{scenario} {format}");
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
