use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;
// , NamedTempFile
use std::path::PathBuf;
use dotenv::dotenv;
// use mp4::{MediaConfig, Mp4Config, Mp4Writer, TrackConfig};
// use std::fs::File;

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


// fn create_test_m4a_file(path: &str, duration_seconds: f32) -> std::io::Result<()> {
//     let sample_rate = 44100;
//     let channels = 1;
//     let samples = (duration_seconds * sample_rate as f32) as u32;

//     let config = Mp4Config {
//         major_brand: mp4::FourCC::from(*b"M4A "),
//         minor_version: 0,
//         compatible_brands: vec![
//             mp4::FourCC::from(*b"M4A "),
//             mp4::FourCC::from(*b"mp42"),
//             mp4::FourCC::from(*b"isom"),
//         ],
//         timescale: sample_rate,
//     };

//     let mut writer = Mp4Writer::write_start(File::create(path)?, &config)?;

//     let track_config = TrackConfig {
//         track_type: mp4::TrackType::Audio,
//         timescale: sample_rate,
//         language: String::from("und"),
//         media_conf: MediaConfig::AacConfig(mp4::AacConfig {
//             bitrate: 128000,
//             profile: mp4::AacProfile::Main,
//             freq_index: mp4::SampleFreqIndex::Freq44100,
//             chan_conf: mp4::ChannelConfig::Mono,
//         }),
//     };

//     let track_id = writer.add_track(&track_config)?;

//     // Generate a simple sine wave
//     let mut audio_data = Vec::new();
//     for i in 0..samples {
//         let t = i as f32 / sample_rate as f32;
//         let sample = ((t * 440.0 * 2.0 * std::f32::consts::PI).sin() * 32767.0) as i16;
//         audio_data.extend_from_slice(&sample.to_le_bytes());
//     }

//     writer.write_sample(
//         track_id,
//         &mp4::Mp4Sample {
//             start_time: 0,
//             duration: samples,
//             rendering_offset: 0,
//             is_sync: true,
//             bytes: mp4::Bytes::from(audio_data),
//         },
//     )?;

//     writer.write_end()?;
//     Ok(())
// }

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

// #[test]
// fn test_valid_file() {
//     let test_env = setup_test_env();
//     let temp_file = NamedTempFile::new().unwrap();
//     let file_path = temp_file.path().to_str().unwrap();

//     // Create a small valid audio file (1 second)
//     create_test_m4a_file(file_path, 1.0).unwrap();

//     let mut cmd = Command::cargo_bin("transcribe").unwrap();
//     cmd.env("TRANSCRIBE_CONFIG_DIR", test_env.config_path.to_str().unwrap())
//         .arg("--input")
//         .arg(file_path)
//         .arg("--is-file")
//         .assert()
//         .success()
//         .stdout(predicate::str::contains("Transcription successful"));
// }

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

// #[test]
// fn test_large_file_upload() {
//     let test_env = setup_test_env();
//     let large_file = NamedTempFile::new().unwrap();
//     let file_path = large_file.path().to_str().unwrap();
    
//    // Create a larger valid audio file (10 seconds)
//     create_test_m4a_file(file_path, 10.0).unwrap();

//     let mut cmd = Command::cargo_bin("transcribe").unwrap();
//     cmd.env("TRANSCRIBE_CONFIG_DIR", test_env.config_path.to_str().unwrap())
//         .arg("--input")
//         .arg(file_path)
//         .arg("--is-file")
//         .assert()
//         .success()
//         .stdout(predicate::str::contains("Transcription successful"));
// }

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