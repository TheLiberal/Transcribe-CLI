use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::{TempDir, NamedTempFile};
use std::path::PathBuf;
use dotenv::dotenv;
use ffmpeg_next as ffmpeg;
use std::f32::consts::PI;

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

fn init_ffmpeg() {
    ffmpeg::init().unwrap();
}

fn create_test_m4a_file(path: &str, duration_seconds: f32) -> Result<(), Box<dyn std::error::Error>> {
    init_ffmpeg();

    let mut oc = ffmpeg::format::output(&path)?;
    let mut stream = oc.add_stream(ffmpeg::encoder::find(ffmpeg::codec::Id::AAC))?;
    let context = stream.codec().encoder().audio()?;
    let mut encoder = context.open_as(ffmpeg::encoder::audio::AudioEncoder)?;

    encoder.set_rate(44100);
    encoder.set_channels(1);
    encoder.set_format(ffmpeg::util::sample::Sample::F32(ffmpeg::util::sample::Type::Packed));
    encoder.set_bit_rate(128_000);

    let time_base = ffmpeg::Rational(1, 44100);
    encoder.set_time_base(time_base);
    stream.set_time_base(time_base);

    oc.write_header()?;

    let mut samples = Vec::new();
    let total_samples = (44100.0 * duration_seconds) as usize;

    for i in 0..total_samples {
        let t = i as f32 / 44100.0;
        let sample = (t * 440.0 * 2.0 * PI).sin();
        samples.push(sample);
    }

    let mut frame = ffmpeg::frame::Audio::new(ffmpeg::format::Sample::F32(ffmpeg::format::sample::Type::Packed), 1024, 1);
    let mut packet = ffmpeg::Packet::empty();

    for chunk in samples.chunks(1024) {
        frame.plane_mut(0).copy_from_slice(chunk);
        frame.set_pts(Some(stream.pts()));

        encoder.send_frame(&frame)?;
        while encoder.receive_packet(&mut packet).is_ok() {
            packet.set_stream(0);
            packet.rescale_ts(time_base, stream.time_base());
            packet.write_interleaved(&mut oc)?;
        }

        stream.set_pts(stream.pts() + 1024);
    }

    encoder.send_eof()?;
    while encoder.receive_packet(&mut packet).is_ok() {
        packet.set_stream(0);
        packet.rescale_ts(time_base, stream.time_base());
        packet.write_interleaved(&mut oc)?;
    }

    oc.write_trailer()?;

    Ok(())
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

    // Create a small valid audio file (1 second)
    create_test_m4a_file(file_path, 1.0).unwrap();

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
    
   // Create a larger valid audio file (10 seconds)
    create_test_m4a_file(file_path, 10.0).unwrap();

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