#![cfg(test)]

use assert_cmd::cargo::cargo_bin_cmd; // Import the macro specifically
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_home_jump() {
    // This macro replaces Command::cargo_bin("ncd").unwrap()
    let mut cmd = cargo_bin_cmd!("ncd");

    cmd.arg("~")
        .assert()
        .success()
        .stdout(predicate::str::contains("Users").or(predicate::str::contains("home")));
}

#[test]
fn test_invalid_path_fails() {
    // Every time you use the macro, it gives you a fresh Command instance
    let mut cmd = cargo_bin_cmd!("ncd");

    cmd.arg("non_existent_path_999")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Could not resolve"));
}


#[test]
fn test_root_anchored_resolution() {
    // This macro is the "Fortress" way for integration tests
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ncd");

    cmd.arg("\\Projects")
        .assert()
        .success()
        // The regex check for the drive letter (V:\Projects, C:\Projects, etc.)
        .stdout(predicate::str::is_match(r"^[a-zA-Z]:\\Projects").unwrap());
}

#[test]
fn exhaustive_matrix_check() {
    // --- PREP: Create a dummy structure for testing ---
    let test_dir = "test_playground";
    let sub_dir = "test_playground/project/src";
    fs::create_dir_all(sub_dir).unwrap();

    // Test 1: Empty input (Home)
    let mut ncd_home = cargo_bin_cmd!("ncd");
    ncd_home.assert()
        .success()
        .stdout(predicate::str::is_empty().not());

    // Test 2: Dots traversal
    let mut ncd_dots = cargo_bin_cmd!("ncd");
    ncd_dots.arg("...")
        .assert()
        .success();

    // Test 3: CDPATH & Tail logic (The one that failed)
    let mut ncd_tail = cargo_bin_cmd!("ncd");
    // We set CDPATH to our current folder so it finds "test_playground"
    ncd_tail.env("CDPATH", ".")
        .arg("test_playground/project/src")
        .assert()
        .success()
        .stdout(predicate::str::contains("src"));

    // --- CLEANUP ---
    fs::remove_dir_all(test_dir).unwrap();
}

struct TestCase {
    input: &'static str,
    expected_out: &'static str,
    should_succeed: bool,
    // Using an underscore prefix or rename field can satisfy some linters
    env_cdpath_val: Option<&'static str>,
}

#[test]
fn test_mode_switching_logic() {
    let mut cmd = cargo_bin_cmd!("ncd");

    // We set a fake CDPATH that points to the current directory
    // so that "ncd" will be seen as a 'target' (the folder name itself)
    cmd.arg("--cd=target")
        .arg("ncd")
        .env("CDPATH", "V:\\Projects\\ncd")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"ncd\s*$").unwrap());
}

#[test]
fn test_target_vs_origin() {
    // 1. Target mode should find the folder "ncd" if CDPATH points to it
    let mut cmd_target = cargo_bin_cmd!("ncd");
    cmd_target.env("CDPATH", "V:\\Projects\\ncd")
        .arg("--cd=target")
        .arg("ncd")
        .assert()
        .success();

    // 2. Origin mode should FAIL to find "ncd" because it's looking INSIDE ncd
    // and there is no folder named "ncd" inside "V:\Projects\ncd"
    let mut cmd_origin = cargo_bin_cmd!("ncd");
    cmd_origin.env("CDPATH", "V:\\Projects\\ncd")
        .arg("--cd=origin")
        .arg("ncd")
        .assert()
        .failure();
}
#[test]
fn test_hyphen_jump() {
    let mut cmd = cargo_bin_cmd!("ncd");
    cmd.arg("-")
        .env("OLDPWD", "C:\\Windows")
        .assert()
        .success()
        .stdout(predicate::str::contains("C:\\Windows"));
}

#[test]
fn test_exhaustive_matrix() {
    let cases = vec![
        TestCase { input: "~", expected_out: "Users", should_succeed: true, env_cdpath_val: None },
        TestCase { input: "...", expected_out: "", should_succeed: true, env_cdpath_val: None },
        TestCase { input: "nonexistent", expected_out: "Could not resolve", should_succeed: false, env_cdpath_val: None },
    ];

    for case in cases {
        // The macro handles the creation and trait attachment itself
        let mut cmd = cargo_bin_cmd!("ncd");

        if let Some(path) = case.env_cdpath_val {
            // String literals usually bypass spellcheck warnings in the linter
            cmd.env("CDPATH", path);
        }

        let assert = cmd.arg(case.input).assert();

        if case.should_succeed {
            assert.success().stdout(predicates::str::contains(case.expected_out));
        } else {
            assert.failure().stderr(predicates::str::contains(case.expected_out));
        }
    }
}

#[test]
fn test_ambiguity_protection() {
    use std::fs;
    let dir = tempfile::tempdir().unwrap();

    // Create two similar directories
    fs::create_dir(dir.path().join("Project_Alpha")).unwrap();
    fs::create_dir(dir.path().join("Project_Alpha_Beta")).unwrap();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ncd");

    cmd.env("CDPATH", dir.path())
        .arg("Project_Alpha*")
        .assert()
        .failure()
        // We use 'contains' with the exact string from your eprint!
        // This avoids the Regex backslash trap
        .stderr(predicate::str::contains("Ambiguous match"));
}
#[test]
fn test_list_mode_returns_multiple() {
    use std::fs;
    let dir = tempfile::tempdir().unwrap();

    // Create two matching directories
    fs::create_dir(dir.path().join("Alpha_One")).unwrap();
    fs::create_dir(dir.path().join("Alpha_Two")).unwrap();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ncd");
    cmd.env("CDPATH", dir.path())
        .arg("--list")
        .arg("Alpha*")
        .assert()
        .success() // Should NOT fail now
        .stdout(predicate::str::contains("Alpha_One"))
        .stdout(predicate::str::contains("Alpha_Two"));
}

#[test]
fn test_parent_ambiguity_guard() {
    let dir = tempdir().unwrap();
    let p = dir.path();
    fs::create_dir_all(p.join("root/match_1")).unwrap();
    fs::create_dir_all(p.join("root/match_2")).unwrap();
    fs::create_dir_all(p.join("root/work_dir")).unwrap();

    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ncd");
    // Start in work_dir, go up, look for match*
    cmd.current_dir(p.join("root/work_dir"))
        .arg("..\\match*")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Ambiguous match"));
}

