use assert_cmd::cargo::cargo_bin_cmd;

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
    assert_eq!(files, ["raw_accounts.csv", "raw_plans.csv"]);
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
