use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn help_explains_defaults_and_prefix_example() {
    let output = cargo_bin_cmd!("rowing-machine")
        .arg("--help")
        .assert()
        .success();
    let help = String::from_utf8_lossy(&output.get_output().stdout);
    for expected in [
        "--years",
        "--scale",
        "--seed",
        "--start-date",
        "--output-dir",
        "--pre",
        "--quiet",
        "--format",
        "--compress",
        "--target-rows",
        "2023-01-01",
        "100",
        "raw_orders.csv",
    ] {
        assert!(help.contains(expected), "missing help: {expected}");
    }
}

#[test]
fn invalid_flags_fail_before_creating_output_and_explain_a_fix() {
    for (flag, value, fix) in [
        ("--years", "0", "positive"),
        ("--years", "-1", "positive"),
        ("--scale", "0", "positive"),
        ("--scale", "no", "positive"),
        ("--target-rows", "0", "positive"),
        ("--target-rows", "-1", "positive"),
        ("--seed", "-1", "unsigned"),
        ("--start-date", "2023-02-29", "YYYY-MM-DD"),
        ("--start-date", "2023-1-1", "YYYY-MM-DD"),
        ("--pre", "../escape", "filename"),
    ] {
        let directory = tempfile::tempdir().unwrap();
        let output_path = directory.path().join("output");
        let result = cargo_bin_cmd!("rowing-machine")
            .args([flag, value, "--output-dir"])
            .arg(&output_path)
            .assert()
            .failure();
        let error = String::from_utf8_lossy(&result.get_output().stderr);
        for expected in [flag, value, fix] {
            assert!(
                error.contains(expected),
                "{error:?} should include {expected}"
            );
        }
        assert!(!error.contains("panicked"));
        assert!(!output_path.exists());
    }
}

#[test]
fn explicit_years_conflict_with_target_rows_before_output() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("output");
    let result = cargo_bin_cmd!("rowing-machine")
        .args(["--years", "4", "--target-rows", "1000", "--output-dir"])
        .arg(&path)
        .assert()
        .failure();
    let error = String::from_utf8_lossy(&result.get_output().stderr);
    assert!(error.contains("--years"), "{error}");
    assert!(error.contains("--target-rows"), "{error}");
    assert!(error.contains("cannot be used"), "{error}");
    assert!(!path.exists());
}

#[test]
fn quiet_generation_writes_csv_without_console_output() {
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
            "--output-dir",
        ])
        .arg(directory.path())
        .assert()
        .success()
        .stdout("")
        .stderr("");
    assert!(directory.path().join("raw_orders.csv").exists());
}

#[test]
fn random_seed_is_printed_and_can_reproduce_the_run() {
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    let result = cargo_bin_cmd!("rowing-machine")
        .args(["--years", "1", "--scale", "1", "--output-dir"])
        .arg(first.path())
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&result.get_output().stdout);
    let seed = stdout
        .lines()
        .find_map(|line| line.strip_prefix("Seed: "))
        .expect("printed seed");
    assert_ne!(seed, "0");
    cargo_bin_cmd!("rowing-machine")
        .args([
            "--years",
            "1",
            "--scale",
            "1",
            "--quiet",
            "--seed",
            seed,
            "--output-dir",
        ])
        .arg(second.path())
        .assert()
        .success();
    for entry in std::fs::read_dir(first.path()).unwrap() {
        let entry = entry.unwrap();
        assert_eq!(
            std::fs::read(entry.path()).unwrap(),
            std::fs::read(second.path().join(entry.file_name())).unwrap()
        );
        let entity = entry
            .file_name()
            .to_string_lossy()
            .trim_start_matches("raw_")
            .trim_end_matches(".csv")
            .to_owned();
        assert!(stdout.contains(&format!("{entity}:")));
    }
    assert!(stdout.contains("Elapsed:"));
}

#[test]
fn out_of_range_store_openings_name_the_start_date_before_writing_files() {
    let directory = tempfile::tempdir().unwrap();
    let result = cargo_bin_cmd!("rowing-machine")
        .args([
            "--years",
            "1",
            "--scale",
            "1",
            "--start-date",
            "9998-01-01",
            "--output-dir",
        ])
        .arg(directory.path())
        .assert()
        .failure();
    let error = String::from_utf8_lossy(&result.get_output().stderr);
    for expected in ["--start-date", "9998-01-01", "earlier"] {
        assert!(error.contains(expected), "{error}");
    }
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 0);
}
