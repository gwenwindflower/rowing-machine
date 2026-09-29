use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn saas_theme_changes_only_names_and_labels_including_derived_emails() {
    let directory = tempfile::tempdir().unwrap();
    let custom = directory.path().join("custom.toml");
    let contents = include_str!("../themes/plain.toml")
        .replace("Adrian", "Aster")
        .replace("Bright", "Radiant")
        .replace("technology", "software")
        .replace("admin", "owner");
    std::fs::write(&custom, contents).unwrap();
    let original = directory.path().join("original");
    let changed = directory.path().join("changed");
    for (output, theme) in [(&original, "plain"), (&changed, custom.to_str().unwrap())] {
        cargo_bin_cmd!("rowing-machine")
            .args([
                "--scenario",
                "saas",
                "--years",
                "1",
                "--scale",
                "4",
                "--seed",
                "42",
                "--quiet",
                "--format",
                "jsonl",
                "--theme",
                theme,
                "--output-dir",
            ])
            .arg(output)
            .assert()
            .success();
    }
    let mut different = false;
    for entry in std::fs::read_dir(&original).unwrap() {
        let entry = entry.unwrap();
        let before = std::fs::read_to_string(entry.path()).unwrap();
        let after = std::fs::read_to_string(changed.join(entry.file_name())).unwrap();
        different |= before != after;
        let normalize = |contents: &str| {
            contents
                .lines()
                .map(|line| {
                    let mut row: serde_json::Value = serde_json::from_str(line).unwrap();
                    for key in ["name", "email", "industry", "region", "role", "tier"] {
                        row.as_object_mut().unwrap().remove(key);
                    }
                    row
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(normalize(&before), normalize(&after));
    }
    assert!(different);
}

#[test]
fn every_saas_format_repeats_byte_identically_with_the_same_seed() {
    for (format, compress) in [
        ("csv", false),
        ("jsonl", false),
        ("parquet", false),
        ("jsonl", true),
        ("parquet", true),
    ] {
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        for directory in [&first, &second] {
            let mut command = cargo_bin_cmd!("rowing-machine");
            command
                .args([
                    "--scenario",
                    "saas",
                    "--years",
                    "1",
                    "--scale",
                    "4",
                    "--seed",
                    "42",
                    "--quiet",
                    "--format",
                    format,
                    "--output-dir",
                ])
                .arg(directory.path());
            if compress {
                command.arg("--compress");
            }
            command.assert().success();
        }
        assert_eq!(std::fs::read_dir(first.path()).unwrap().count(), 6);
        for entry in std::fs::read_dir(first.path()).unwrap() {
            let entry = entry.unwrap();
            assert_eq!(
                std::fs::read(entry.path()).unwrap(),
                std::fs::read(second.path().join(entry.file_name())).unwrap(),
                "{format}, compressed={compress}"
            );
        }
    }
}

#[test]
fn account_calibration_respects_population_and_preserves_output_integrity() {
    let directory = tempfile::tempdir().unwrap();
    cargo_bin_cmd!("rowing-machine")
        .args([
            "--scenario",
            "saas",
            "--scale",
            "4",
            "--target-rows",
            "40",
            "--seed",
            "42",
            "--quiet",
            "--output-dir",
        ])
        .arg(directory.path())
        .assert()
        .success();
    let count = csv::Reader::from_path(directory.path().join("raw_accounts.csv"))
        .unwrap()
        .records()
        .count();
    assert!((38..=42).contains(&count), "{count}");
    let rejected = directory.path().join("too-many");
    let result = cargo_bin_cmd!("rowing-machine")
        .args([
            "--scenario",
            "saas",
            "--scale",
            "4",
            "--target-rows",
            "81",
            "--output-dir",
        ])
        .arg(&rejected)
        .assert()
        .failure();
    let error = String::from_utf8_lossy(&result.get_output().stderr);
    assert!(
        error.contains("--target-rows 81") && error.contains("--scale"),
        "{error}"
    );
    assert!(!rejected.exists());
}

#[test]
fn saas_selects_its_entities_and_rejects_incompatible_themes_before_output() {
    let directory = tempfile::tempdir().unwrap();
    cargo_bin_cmd!("rowing-machine")
        .args([
            "--scenario",
            "saas",
            "--years",
            "1",
            "--scale",
            "4",
            "--seed",
            "42",
            "--quiet",
            "--output-dir",
        ])
        .arg(directory.path())
        .assert()
        .success();
    let mut files: Vec<_> = std::fs::read_dir(directory.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    files.sort();
    assert_eq!(
        files,
        [
            "raw_accounts.csv",
            "raw_invoices.csv",
            "raw_mrr_movements.csv",
            "raw_plans.csv",
            "raw_subscriptions.csv",
            "raw_users.csv"
        ]
    );
    let rejected = directory.path().join("rejected");
    let result = cargo_bin_cmd!("rowing-machine")
        .args([
            "--scenario",
            "saas",
            "--theme",
            "fantasy_rpg",
            "--output-dir",
        ])
        .arg(&rejected)
        .assert()
        .failure();
    let error = String::from_utf8_lossy(&result.get_output().stderr);
    assert!(
        error.contains("organization") && error.contains("plain"),
        "{error}"
    );
    assert!(!rejected.exists());
}
