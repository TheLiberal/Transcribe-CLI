use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::{TempDir, NamedTempFile};
use std::path::PathBuf;
use dotenv::dotenv;

struct TestEnv {
    _temp_dir: TempDir,
    config_path: PathBuf,
}

fn setup_test_env() -> TestEnv {
    dotenv().ok();
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join(".transcribe_cli");
    fs::create_dir_all(&config_path).unwrap();
    
    let api_key = std::env::var("DEEPGRAM_API_KEY")
        .expect("DEEPGRAM_API_KEY must be set for tests");
    fs::write(config_path.join("api_key"), &api_key).unwrap();
    
    TestEnv {
        _temp_dir: temp_dir,
        config_path,
    }
}

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
    let test_env = setup_test_env();
    let mut cmd = Command::cargo_bin("transcribe").unwrap();
    cmd.env("TRANSCRIBE_CONFIG_DIR", test_env.config_path.to_str().unwrap())
        .arg("--input")
        .arg("not_a_url")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid URL provided"));
}

#[test]
fn test_nonexistent_file() {
    let test_env = setup_test_env();
    let mut cmd = Command::cargo_bin("transcribe").unwrap();
    cmd.env("TRANSCRIBE_CONFIG_DIR", test_env.config_path.to_str().unwrap())
        .arg("--input")
        .arg("nonexistent_file.mp3")
        .arg("--is-file")
        .assert()
        .failure()
        .stderr(predicate::str::contains("The system cannot find the file specified"));
}

#[test]
fn test_valid_url() {
    let test_env = setup_test_env();
    let mut cmd = Command::cargo_bin("transcribe").unwrap();
    cmd.env("TRANSCRIBE_CONFIG_DIR", test_env.config_path.to_str().unwrap())
        .arg("--input")
        .arg("https://utfs.io/f/70e48391-743c-4b3f-a42e-c48175fdbcca-7ontaw.m4a")
        .assert()
        .success()
        .stdout(predicate::str::contains("Transcription successful"));
}

#[test]
fn test_valid_file() {
    let test_env = setup_test_env();
    let temp_file = NamedTempFile::new().unwrap();
    let file_path = temp_file.path().to_str().unwrap();

    // Write some valid audio data to the file
    std::fs::write(file_path, include_bytes!("C:/Users/Blaise/Downloads/firewater_mark_evan.m4a")).unwrap();

    let mut cmd = Command::cargo_bin("transcribe").unwrap();
    cmd.env("TRANSCRIBE_CONFIG_DIR", test_env.config_path.to_str().unwrap())
        .arg("--input")
        .arg(file_path)
        .arg("--is-file")
        .assert()
        .success()
        .stdout(predicate::str::contains("Transcription successful"));
}

#[test]
fn test_api_key_prompt() {
    let test_env = setup_test_env();
    let api_key_path = test_env.config_path.join("api_key");
    if api_key_path.exists() {
        fs::remove_file(&api_key_path).unwrap();
    }

    let valid_api_key = std::env::var("DEEPGRAM_API_KEY")
        .expect("DEEPGRAM_API_KEY must be set for tests");

    let mut cmd = Command::cargo_bin("transcribe").unwrap();
    cmd.env("TRANSCRIBE_CONFIG_DIR", test_env.config_path.to_str().unwrap())
        .arg("--input")
        .arg("https://utfs.io/f/70e48391-743c-4b3f-a42e-c48175fdbcca-7ontaw.m4a")
        .write_stdin(format!("{}\n", valid_api_key))
        .assert()
        .success()
        .stdout(predicate::str::contains("API key saved"));

    assert!(api_key_path.exists());
}

#[test]
fn test_invalid_api_key() {
    let test_env = setup_test_env();
    fs::write(test_env.config_path.join("api_key"), "invalid_key_for_testing").unwrap();

    let mut cmd = Command::cargo_bin("transcribe").unwrap();
    cmd.env("TRANSCRIBE_CONFIG_DIR", test_env.config_path.to_str().unwrap())
        .arg("--input")
        .arg("https://utfs.io/f/70e48391-743c-4b3f-a42e-c48175fdbcca-7ontaw.m4a")
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid API key"));
}

#[test]
fn test_network_error() {
    let test_env = setup_test_env();
    let mut cmd = Command::cargo_bin("transcribe").unwrap();
    cmd.env("TRANSCRIBE_CONFIG_DIR", test_env.config_path.to_str().unwrap())
        .arg("--input")
        .arg("https://nonexistent-url.com/audio.mp3")
        .assert()
        .failure()
        .stderr(predicate::str::contains("API request failed"));
}

#[test]
fn test_large_file_upload() {
    let test_env = setup_test_env();
    let large_file = NamedTempFile::new().unwrap();
    let file_path = large_file.path().to_str().unwrap();
    
    // Create a large file with valid audio data
    let large_data = include_bytes!("C:/Users/Blaise/Downloads/firewater_mark_evan.m4a");
    fs::write(file_path, large_data).unwrap();

    let mut cmd = Command::cargo_bin("transcribe").unwrap();
    cmd.env("TRANSCRIBE_CONFIG_DIR", test_env.config_path.to_str().unwrap())
        .arg("--input")
        .arg(file_path)
        .arg("--is-file")
        .assert()
        .success()
        .stdout(predicate::str::contains("Transcription successful"));
}

#[test]
fn test_output_file_creation() {
    let test_env = setup_test_env();
    
    // Create a temporary directory for output
    let temp_output_dir = tempfile::tempdir().unwrap();
    
    let mut cmd = Command::cargo_bin("transcribe").unwrap();
    cmd.env("TRANSCRIBE_CONFIG_DIR", test_env.config_path.to_str().unwrap())
        .env("TRANSCRIBE_OUTPUT_DIR", temp_output_dir.path())
        .arg("--input")
        .arg("https://utfs.io/f/70e48391-743c-4b3f-a42e-c48175fdbcca-7ontaw.m4a")
        .assert()
        .success()
        .stdout(predicate::str::contains("Transcription successful"));

    // Check if a file was created in the temporary output directory
    let files = fs::read_dir(temp_output_dir.path()).unwrap();
    assert!(files.filter_map(Result::ok)
                 .any(|f| f.file_name().to_str().unwrap().starts_with("transcription-")));
}