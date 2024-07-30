use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;

#[test]
fn test_help() {
    let mut cmd = Command::cargo_bin("transcribe").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("USAGE:"));
}

#[test]
fn test_invalid_url() {
    let mut cmd = Command::cargo_bin("transcribe").unwrap();
    cmd.arg("--input")
        .arg("not_a_url")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid URL provided"));
}

#[test]
fn test_nonexistent_file() {
    let mut cmd = Command::cargo_bin("transcribe").unwrap();
    cmd.arg("--input")
        .arg("nonexistent_file.mp3")
        .arg("--is-file")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No such file or directory"));
}

// Add more tests as needed