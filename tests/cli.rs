use assert_cmd::cargo::cargo_bin_cmd;

#[test]
fn themes_lists_bundled_descriptions_and_compatible_scenarios_without_generating() {
    let directory = tempfile::tempdir().unwrap();
    let result = cargo_bin_cmd!("rowing-machine")
        .current_dir(directory.path())
        .arg("themes")
        .assert()
        .success();
    let stdout = String::from_utf8_lossy(&result.get_output().stdout);
    for name in ["plain", "fantasy_rpg"] {
        let line = stdout.lines().find(|line| line.starts_with(name)).unwrap();
        assert!(line.contains("ecommerce"));
        assert!(line.len() > name.len() + "ecommerce".len() + 4);
    }
    assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 0);
}

#[test]
fn bad_theme_files_report_the_path_and_field_before_creating_output() {
    for (contents, field) in [
        ("name = [", "name"),
        (
            "name = 'broken'\ndescription = 'Incomplete theme'",
            "person",
        ),
    ] {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("broken.toml");
        std::fs::write(&path, contents).unwrap();
        let output = directory.path().join("output");
        let result = cargo_bin_cmd!("rowing-machine")
            .args(["--theme"])
            .arg(&path)
            .arg("--output-dir")
            .arg(&output)
            .assert()
            .failure();
        let error = String::from_utf8_lossy(&result.get_output().stderr);
        assert!(error.contains(path.to_str().unwrap()), "{error}");
        assert!(error.contains(field), "{error}");
        assert!(!output.exists());
    }
}

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
        "--theme",
        "--workers",
        "fantasy_rpg",
        "plain",
        "2023-01-01",
        "100",
        "raw_orders.csv",
    ] {
        assert!(help.contains(expected), "missing help: {expected}");
    }
}

#[test]
fn bundled_and_path_themes_write_identical_files_and_plain_is_the_default() {
    for (name, contents) in [
        ("plain", include_str!("../themes/plain.toml")),
        ("fantasy_rpg", include_str!("../themes/fantasy_rpg.toml")),
    ] {
        let directory = tempfile::tempdir().unwrap();
        let theme_path = directory.path().join("theme.toml");
        std::fs::write(&theme_path, contents).unwrap();
        let selectors = [Some(name), Some(theme_path.to_str().unwrap())];
        for (index, selector) in selectors.into_iter().enumerate() {
            let mut command = cargo_bin_cmd!("rowing-machine");
            command.args(["--years", "1", "--scale", "1", "--seed", "42", "--quiet"]);
            if let Some(selector) = selector {
                command.args(["--theme", selector]);
            }
            command
                .arg("--output-dir")
                .arg(directory.path().join(index.to_string()))
                .assert()
                .success();
        }
        for entry in std::fs::read_dir(directory.path().join("0")).unwrap() {
            let entry = entry.unwrap();
            assert_eq!(
                std::fs::read(entry.path()).unwrap(),
                std::fs::read(directory.path().join("1").join(entry.file_name())).unwrap()
            );
        }
        if name == "plain" {
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
                .arg(directory.path().join("default"))
                .assert()
                .success();
            for entry in std::fs::read_dir(directory.path().join("0")).unwrap() {
                let entry = entry.unwrap();
                assert_eq!(
                    std::fs::read(entry.path()).unwrap(),
                    std::fs::read(directory.path().join("default").join(entry.file_name()))
                        .unwrap()
                );
            }
        }
    }
}

#[test]
fn incompatible_theme_reports_lengths_and_compatible_choices_before_output() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("short.toml");
    let contents = include_str!("../themes/plain.toml").replace(
        "ranks = [\"new\", \"regular\", \"loyal\", \"ambassador\"]",
        "ranks = [\"member\"]",
    );
    std::fs::write(&path, contents).unwrap();
    let output = directory.path().join("output");
    let result = cargo_bin_cmd!("rowing-machine")
        .arg("--theme")
        .arg(&path)
        .arg("--output-dir")
        .arg(&output)
        .assert()
        .failure();
    let error = String::from_utf8_lossy(&result.get_output().stderr);
    for expected in [
        "--theme",
        "short.toml",
        "ecommerce",
        "labels.ranks",
        "expected 4",
        "plain",
        "fantasy_rpg",
    ] {
        assert!(error.contains(expected), "{error}");
    }
    assert!(!output.exists());
}

#[test]
fn invalid_flags_fail_before_creating_output_and_explain_a_fix() {
    for (flag, value, fix) in [
        ("--years", "0", "positive"),
        ("--workers", "0", "positive"),
        ("--workers", "-1", "positive"),
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
