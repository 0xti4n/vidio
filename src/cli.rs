use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "vidio")]
#[command(about = "Vidio Transcript Downloader and Analyzer")]
#[command(version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Force CLI mode (skip TUI)
    #[arg(long)]
    pub cli: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Download transcript and optionally generate report
    Get {
        /// YouTube video URL or video ID
        video_id: String,

        /// Preferred languages (comma-separated)
        #[arg(short, long, default_value = "en,es")]
        languages: String,

        /// Preserve formatting in transcript
        #[arg(long)]
        preserve_formatting: bool,

        /// Generate report after downloading transcript
        #[arg(short, long)]
        report: bool,
    },

    /// Generate report from existing transcript
    Report {
        /// Video ID of existing transcript
        video_id: String,
    },

    /// List all downloaded transcripts and reports
    List,

    /// Open TUI interface
    Tui,
}
