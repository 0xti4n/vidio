mod cli;
mod core;
mod error;
mod tui;

use crate::cli::{Cli, Commands};
use crate::core::{ReportService, StorageService, TranscriptService, extract_video_id};
use crate::error::Result;
use crate::tui::{App, EventHandler, init as tui_init, restore as tui_restore, ui};
use clap::Parser;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Get {
            video_id,
            languages,
            preserve_formatting,
            report,
        }) => {
            run_cli_get(video_id, languages, preserve_formatting, report).await?;
        }
        Some(Commands::Report { video_id }) => {
            run_cli_report(video_id).await?;
        }
        Some(Commands::List) => {
            run_cli_list()?;
        }
        Some(Commands::Tui) | None => {
            if cli.cli {
                println!("Use 'ytranscript --help' for available commands");
            } else {
                run_tui().await?;
            }
        }
    }

    Ok(())
}

async fn run_cli_get(
    video_input: String,
    languages: String,
    preserve_formatting: bool,
    generate_report: bool,
) -> Result<()> {
    let video_id = extract_video_id(&video_input)
        .ok_or_else(|| error::Error::custom("Invalid video URL or ID"))?;

    println!("Processing video: {video_id}");

    let transcript_service = TranscriptService::new()?;
    let report_service = ReportService::new();

    let languages: Vec<&str> = languages.split(',').map(|s| s.trim()).collect();

    // Fetch transcript
    println!("Fetching transcript...");
    let transcript = transcript_service
        .fetch_transcript(&video_id, &languages, preserve_formatting)
        .await?;

    // Save transcript
    let transcript_path = StorageService::save_transcript(&transcript)?;
    println!("Transcript saved to: {transcript_path:?}");

    // Generate report if requested
    if generate_report {
        println!("Generating report...");
        let report_content = report_service.generate_report(&transcript).await?;
        let report_path = StorageService::save_report(&video_id, &report_content)?;
        println!("Report saved to: {report_path:?}");
    }

    Ok(())
}

async fn run_cli_report(video_id: String) -> Result<()> {
    println!("Generating report for video: {video_id}");

    let transcript_content = StorageService::load_transcript(&video_id)?;

    let report_service = ReportService::new();
    let report_content = report_service
        .generate_report_text(&transcript_content)
        .await?;

    let report_path = StorageService::save_report(&video_id, &report_content)?;
    println!("Report saved to: {report_path:?}");

    Ok(())
}

fn run_cli_list() -> Result<()> {
    let files = StorageService::list_files()?;

    if files.is_empty() {
        println!("No files found.");
        return Ok(());
    }

    println!("Found {} files:", files.len());
    println!();

    for file in files {
        let file_type = match file.file_type {
            core::storage::FileType::Transcript => "Transcript",
            core::storage::FileType::Report => "Report",
        };

        let size_kb = file.size / 1024;
        let size_str = if size_kb < 1024 {
            format!("{size_kb}KB")
        } else {
            format!("{:.1}MB", size_kb as f64 / 1024.0)
        };

        println!("{:<12} {:<30} {}", file_type, file.name, size_str);
    }

    Ok(())
}

async fn run_tui() -> Result<()> {
    // Initialize terminal
    let mut terminal = tui_init()?;

    // Create app
    let mut app = App::new()?;
    let event_handler = EventHandler::new();

    // Setup async communication channel for background tasks
    let (tx, rx) = mpsc::unbounded_channel();
    app.processing_tx = Some(tx.clone());
    app.processing_rx = Some(rx);

    // Main event loop
    loop {
        // Handle events
        let event = event_handler.next_event()?;
        app.handle_event(event)?;

        // Draw UI
        terminal.draw(|f| {
            ui::draw(f, &mut app);
        })?;

        // Check if we should quit
        if app.should_quit {
            break;
        }

        // Background processing is now handled in app.rs via real async tasks
        // No additional processing needed here
    }

    // Restore terminal
    tui_restore()?;
    Ok(())
}
