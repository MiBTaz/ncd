use std::env;
use assert_cmd::cargo::cargo_bin_cmd; // Import the macro specifically
use predicates::prelude::*;

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
fn test_mode_switching_logic() {
    // Test that 'target' mode finds the bookmark itself, not its contents
    let mut cmd = cargo_bin_cmd!("ncd");
    cmd.arg("--cd=target").arg("Projects")
        .assert()
        .success()
        .stdout(predicate::str::is_match(r"^[a-zA-Z]:\\Projects\s*$").unwrap());
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

