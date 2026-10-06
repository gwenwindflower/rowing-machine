use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn calibrated_jsonl_preserves_orders_across_themes_and_uses_selected_labels() {
    let plain = tempfile::tempdir().unwrap();
    let fantasy = tempfile::tempdir().unwrap();
    for (directory, theme, store) in [
        (&plain, "plain", "Central"),
        (&fantasy, "fantasy_rpg", "Thornwall"),
    ] {
        cargo_bin_cmd!("rowing-machine")
            .args([
                "--target-rows",
                "3000",
                "--scale",
                "1",
                "--seed",
                "42",
                "--quiet",
                "--format",
                "jsonl",
                "--theme",
                theme,
                "--output-dir",
            ])
            .arg(directory.path())
            .assert()
            .success()
            .stdout("")
            .stderr("");
        let stores = std::fs::read_to_string(directory.path().join("raw_stores.jsonl")).unwrap();
        let first: serde_json::Value =
            serde_json::from_str(stores.lines().next().unwrap()).unwrap();
        assert_eq!(first["name"], store);
    }
    let orders = std::fs::read_to_string(plain.path().join("raw_orders.jsonl")).unwrap();
    assert!(orders.lines().count().abs_diff(3_000) <= 150);
    assert_eq!(
        orders,
        std::fs::read_to_string(fantasy.path().join("raw_orders.jsonl")).unwrap()
    );
}

#[test]
fn calibrated_ecommerce_output_lands_within_five_percent_of_the_target() {
    let directory = tempfile::tempdir().unwrap();
    let output = cargo_bin_cmd!("rowing-machine")
        .args([
            "--target-rows",
            "3000",
            "--scale",
            "1",
            "--seed",
            "42",
            "--output-dir",
        ])
        .arg(directory.path())
        .assert()
        .success();
    let stderr = String::from_utf8_lossy(&output.get_output().stderr);
    assert!(stderr.contains("Calibrating orders"), "{stderr}");
    let count = csv::Reader::from_path(directory.path().join("raw_orders.csv"))
        .unwrap()
        .records()
        .map(Result::unwrap)
        .count();
    assert!(count.abs_diff(3_000) <= 150, "generated {count} orders");
}

#[test]
fn seeded_calibration_produces_byte_identical_output() {
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    for directory in [&first, &second] {
        cargo_bin_cmd!("rowing-machine")
            .args([
                "--target-rows",
                "1000",
                "--scale",
                "1",
                "--seed",
                "42",
                "--quiet",
                "--output-dir",
            ])
            .arg(directory.path())
            .assert()
            .success();
    }
    for entry in std::fs::read_dir(first.path()).unwrap() {
        let entry = entry.unwrap();
        assert_eq!(
            std::fs::read(entry.path()).unwrap(),
            std::fs::read(second.path().join(entry.file_name())).unwrap()
        );
    }
}
