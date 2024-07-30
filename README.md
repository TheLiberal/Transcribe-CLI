# Transcribe-CLI

![Rust](https://github.com/theliberal/transcribe-cli/workflows/Rust/badge.svg)

## 🎙️ Effortless Audio Transcription at Your Fingertips

Transcribe CLI is a powerful, user-friendly command-line tool designed to simplify the process of transcribing audio from various sources. Born out of the need to quickly and efficiently transcribe client calls, team meetings, and other audio content throughout the day, this tool handles both local files and remote URLs with ease.

### 🌟 Key Features

- Transcribe local audio files or remote URLs
- Utilizes Deepgram's advanced AI for accurate transcriptions
- Outputs beautifully formatted Markdown files
- Progress bars for real-time feedback
- Automatic retries for improved reliability
- Secure API key management

## 🚀 Getting Started

### Prerequisites

- Rust programming environment
- Deepgram API key

### Installation

1. Clone the repository:

   ```
   git clone https://github.com/yourusername/transcribe-cli.git
   ```

2. Build the project:

   ```
   cd transcribe-cli
   cargo build --release
   ```

3. Run the tool:

   ```
   ./target/release/transcribe-cli -i <input_file_or_url> [-f]
   ```

   Use the `-f` flag when transcribing a local file.

## 🛠️ Usage

### Transcribe a local file:

```
transcribe-cli -i /path/to/audio/file.mp3 -f

```

### Transcribe from a URL:

```
transcribe-cli -i https://example.com/audio/file.mp3

```

## 🔐 API Key Management

On first use, you'll be prompted to enter your Deepgram API key. The key is securely stored in `~/.transcribe_cli/api_key` for future use.

## 📄 Output

Transcriptions are saved as Markdown files on your Desktop, named with the current date and time for easy reference.

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guidelines](CONTRIBUTING.md) for more details.

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Deepgram](https://www.deepgram.com/) for their excellent transcription API

---

Built with ❤️ by Blaise Gulaj
