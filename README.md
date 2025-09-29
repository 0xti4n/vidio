# YTranscript

A powerful YouTube transcript downloader and analyzer built in Rust, featuring both CLI and TUI interfaces with AI-powered content analysis.

## Features

### 🎯 Core Functionality
- **YouTube Transcript Download**: Extract transcripts from YouTube videos using video URLs or IDs
- **Multi-language Support**: Download transcripts in multiple languages (English, Spanish, etc.)
- **AI-Powered Analysis**: Generate detailed content reports using OpenAI's GPT-4 model
- **Dual Interface**: Choose between Command Line Interface (CLI) or Terminal User Interface (TUI)

### 📊 Advanced Analysis
The AI report generation provides comprehensive analysis including:
- **Metadata extraction** (duration, language, speakers)
- **Chronological section indexing**
- **Line-by-line breakdown** with timestamps and tone analysis
- **Entity recognition** (people, brands, places mentioned)
- **Key quotes extraction** (15+ words)
- **Call-to-action identification**
- **Keyword frequency analysis**
- **Rhetorical structure analysis**

### 🖥️ User Interfaces

#### CLI Mode
- `get`: Download transcripts and optionally generate reports
- `report`: Generate reports from existing transcripts
- `list`: View all downloaded files
- `tui`: Launch the interactive terminal interface

#### TUI Mode
- Interactive terminal interface with navigation
- File browser with filtering and search
- Content viewer for transcripts and reports
- Progress tracking for downloads and processing
- Settings configuration

## Installation

### Prerequisites
- Rust 1.70+ (2024 edition)
- OpenAI API key (for report generation)

### From Source
```bash
git clone https://github.com/yourusername/ytranscript.git
cd ytranscript
cargo build --release
```

### Environment Setup
Set your OpenAI API key:
```bash
export OPENAI_API_KEY="your-api-key-here"
# Explicitly opt in before sending transcripts to OpenAI for report generation
export YTRANSCRIPT_ALLOW_OPENAI=1
```

## Usage

### CLI Examples

#### Download a transcript
```bash
ytranscript get "https://youtu.be/VIDEO_ID"
```

#### Download with specific languages and generate report
```bash
ytranscript get "https://youtu.be/VIDEO_ID" --languages "en,es" --report
```

#### Generate report from existing transcript
```bash
ytranscript report VIDEO_ID
```

#### List all files
```bash
ytranscript list
```

### TUI Mode
Launch the interactive terminal interface:
```bash
ytranscript tui
# or simply
ytranscript
```

Navigate through the TUI using:
- **Arrow keys**: Navigate menus and lists
- **Enter**: Select options
- **Tab**: Switch between input fields
- **Esc**: Go back or quit
- **q**: Quit application

## Project Structure

```
ytranscript/
├── src/
│   ├── main.rs           # Application entry point
│   ├── cli.rs            # Command-line interface definitions
│   ├── error.rs          # Error handling
│   ├── core/             # Core business logic
│   │   ├── mod.rs
│   │   ├── transcript.rs # YouTube transcript fetching
│   │   ├── report.rs     # AI report generation
│   │   └── storage.rs    # File storage management
│   └── tui/              # Terminal User Interface
│       ├── mod.rs
│       ├── app.rs        # TUI application state
│       ├── ui.rs         # UI rendering
│       ├── events.rs     # Event handling
│       └── components/   # UI components
│           ├── input.rs
│           ├── list.rs
│           ├── progress.rs
│           └── viewer.rs
├── Cargo.toml
└── README.md
```

## Dependencies

### Core Dependencies
- **yt-transcript-rs**: YouTube transcript API client
- **tokio**: Async runtime
- **async-openai**: OpenAI API client for report generation
- **clap**: Command-line argument parsing

### TUI Dependencies
- **ratatui**: Terminal user interface framework
- **crossterm**: Cross-platform terminal manipulation

### Utility Dependencies
- **serde**: Serialization/deserialization
- **chrono**: Date and time handling
- **derive_more**: Derive macro utilities

## File Output

### Transcripts
- **Format**: Plain text files
- **Naming**: `transcript_{VIDEO_ID}.txt`
- **Content**: Raw transcript text with timestamps (if available)

### Reports
- **Format**: Markdown files
- **Naming**: `report_{VIDEO_ID}.md`
- **Content**: Comprehensive AI-generated analysis including:
  - Metadata table
  - Chronological index
  - Line-by-line breakdown
  - Entity mentions
  - Key quotes
  - Call-to-actions
  - Keyword analysis
  - Executive summary

## Configuration

The application supports various configuration options:

- **Languages**: Specify preferred transcript languages (comma-separated)
- **Preserve Formatting**: Maintain original transcript formatting
- **Report Generation**: Enable/disable AI report generation
- **File Filtering**: Filter files by type (transcripts/reports)

## Error Handling

The application includes comprehensive error handling for:
- Invalid YouTube URLs or video IDs
- Network connectivity issues
- API rate limiting
- File system operations
- OpenAI API errors

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.


## Roadmap

- [ ] Support for batch processing multiple videos
- [ ] Export reports in multiple formats (PDF, HTML)
- [ ] Custom AI prompt templates
- [ ] Transcript search and filtering
- [ ] Integration with other video platforms
- [ ] Web interface option
