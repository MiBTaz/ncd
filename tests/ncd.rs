#![cfg(test)]

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_home_jump() {
    let mut cmd = cargo_bin_cmd!("ncd");
    cmd.arg("~")
        .assert()
        .success()
        // Broad check for cross-platform home directory naming
        .stdout(predicate::str::contains("Users").or(predicate::str::contains("home")));
}

#[test]
fn test_invalid_path_fails() {
    let mut cmd = cargo_bin_cmd!("ncd");
    cmd.arg("non_existent_path_999")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Could not resolve"));
}

#[test]
fn test_root_anchored_resolution() {
    let mut cmd = cargo_bin_cmd!("ncd");
    cmd.arg("\\Projects")
        .assert()
        .success()
        // Matches drive letter root or unix root
        .stdout(predicate::str::is_match(r"([a-zA-Z]:)?\\Projects").unwrap());
}

#[test]
fn exhaustive_matrix_check() {
    // Use tempdir for automatic cleanup even if the test panics
    let dir = tempdir().unwrap();
    let playground = dir.path().join("test_playground");
    let sub_dir = playground.join("project/src");
    fs::create_dir_all(&sub_dir).unwrap();

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

    // Test 3: CDPATH & Tail logic
    let mut ncd_tail = cargo_bin_cmd!("ncd");
    ncd_tail.env("CDPATH", dir.path()) // Set CDPATH to the temp root
        .arg("test_playground/project/src")
        .assert()
        .success()
        .stdout(predicate::str::contains("src"));
}

struct TestCase {
    input: &'static str,
    expected_out: &'static str,
    should_succeed: bool,
    env_cdpath_val: Option<String>,
}

#[test]
fn test_mode_switching_logic() {
    let mut cmd = cargo_bin_cmd!("ncd");
    let current_dir = std::env::current_dir().unwrap();

    cmd.arg("--cd=target")
        .arg("ncd")
        .env("CDPATH", current_dir.parent().unwrap())
        .assert()
        .success()
        .stdout(predicate::str::contains("ncd"));
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
        TestCase { input: "nonexistent_abc_123", expected_out: "Could not resolve", should_succeed: false, env_cdpath_val: None },
    ];

    for case in cases {
        let mut cmd = cargo_bin_cmd!("ncd");
        if let Some(path) = case.env_cdpath_val {
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
    let dir = tempdir().unwrap();
    fs::create_dir(dir.path().join("Project_Alpha")).unwrap();
    fs::create_dir(dir.path().join("Project_Alpha_Beta")).unwrap();

    let mut cmd = cargo_bin_cmd!("ncd");
    cmd.env("CDPATH", dir.path())
        .arg("Project_Alpha*")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Ambiguous match"));
}

#[test]
fn test_list_mode_returns_multiple() {
    let dir = tempdir().unwrap();
    fs::create_dir(dir.path().join("Alpha_One")).unwrap();
    fs::create_dir(dir.path().join("Alpha_Two")).unwrap();

    let mut cmd = cargo_bin_cmd!("ncd");
    cmd.env("CDPATH", dir.path())
        .arg("--list")
        .arg("Alpha*")
        .assert()
        .success()
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

    let mut cmd = cargo_bin_cmd!("ncd");
    cmd.current_dir(p.join("root/work_dir"))
        .arg("..\\match*")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Ambiguous match"));
}

#[test]
fn test_parent_glob_isolation() {
    let dir = tempdir().unwrap();
    let root = dir.path();

    // Create: root/neighbor_target
    // Create: root/current_work_dir/neighbor_target (This is the distractor!)
    let neighbor = root.join("neighbor_target");
    let work_dir = root.join("current_work_dir");
    let distractor = work_dir.join("neighbor_target");

    fs::create_dir_all(&neighbor).unwrap();
    fs::create_dir_all(&distractor).unwrap();

    let mut cmd = cargo_bin_cmd!("ncd");
    // We are INSIDE current_work_dir.
    // We want to jump to ..\neigh* (which should be root\neighbor_target)
    // If the bug exists, it will find BOTH neighbor_target AND current_work_dir\neighbor_target
    cmd.current_dir(&work_dir)
        .arg("..\\neigh*")
        .assert()
        .success()
        .stdout(predicate::str::contains("current_work_dir").not()) // Should NOT see the local one
        .stdout(predicate::str::contains("neighbor_target"));
}
