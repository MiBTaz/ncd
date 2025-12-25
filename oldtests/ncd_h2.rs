use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;

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