use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_help() {
    let mut cmd = Command::cargo_bin("transcribe").unwrap();
   cmd.arg("--help")
    .assert()
    .success()
    .stdout(predicate::str::contains("Usage:"));
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
        .stderr(predicate::str::contains("The system cannot find the file specified"));
}

#[test]
fn test_valid_url() {
    let mut cmd = Command::cargo_bin("transcribe").unwrap();
    cmd.arg("--input")
        .arg("https://utfs.io/f/70e48391-743c-4b3f-a42e-c48175fdbcca-7ontaw.m4a")
        .assert()
        .success()
        .stdout(predicate::str::contains("Transcription successful"));
}

#[test]
fn test_valid_file() {
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_str().unwrap();

    // Write some valid audio data to the file
    std::fs::write(file_path, include_bytes!("C:/Users/Blaise/Downloads/firewater_mark_evan.m4a")).unwrap();


    let mut cmd = Command::cargo_bin("transcribe").unwrap();
    cmd.arg("--input")
        .arg(file_path)
        .arg("--is-file")
        .assert()
        .success()
        .stdout(predicate::str::contains("Transcription successful"));
}

#[test]
fn test_api_key_prompt() {
    let mut cmd = Command::cargo_bin("transcribe").unwrap();
    cmd.arg("--input")
        .arg("https://utfs.io/f/70e48391-743c-4b3f-a42e-c48175fdbcca-7ontaw.m4a")
        .write_stdin("test_api_key\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("API key saved"));
}

#[test]
fn test_invalid_api_key() {
    let mut cmd = Command::cargo_bin("transcribe").unwrap();
    cmd.arg("--input")
        .arg("https://utfs.io/f/70e48391-743c-4b3f-a42e-c48175fdbcca-7ontaw.m4a")
        .env("DEEPGRAM_API_KEY", "invalid_key")
        .assert()
        .failure()
        .stderr(predicate::str::contains("API request failed"));
}

#[test]
fn test_network_error() {
    let mut cmd = Command::cargo_bin("transcribe").unwrap();
    cmd.arg("--input")
        .arg("https://utfs.io/f/70e48391-743c-4b3f-a42e-c48175fdbcca-7ontaw.m4a")
        .assert()
        .failure()
        .stderr(predicate::str::contains("API request failed"));
}

#[test]
fn test_large_file_upload() {
    let large_file = NamedTempFile::new().unwrap();
    let file_path = large_file.path().to_str().unwrap();
    
    // Create a large file (e.g., 100MB)
    // let large_data = vec![0u8; 100 * 1024 * 1024];
    let large_data = include_bytes!("C:/Users/Blaise/Downloads/firewater_mark_evan.m4a");
    fs::write(file_path, large_data).unwrap();

    let mut cmd = Command::cargo_bin("transcribe").unwrap();
    cmd.arg("--input")
        .arg(file_path)
        .arg("--is-file")
        .assert()
        .success()
        .stdout(predicate::str::contains("Transcription successful"));
}

#[test]
fn test_output_file_creation() {
    let mut cmd = Command::cargo_bin("transcribe").unwrap();
    cmd.arg("--input")
        .arg("https://utfs.io/f/70e48391-743c-4b3f-a42e-c48175fdbcca-7ontaw.m4a")
        .assert()
        .success();

    // Check if a file was created in the desktop directory
    let desktop = dirs::desktop_dir().expect("Could not find desktop directory");
    let files = fs::read_dir(desktop).unwrap();
    assert!(files.filter_map(Result::ok)
                 .any(|f| f.file_name().to_str().unwrap().starts_with("transcription-")));
}