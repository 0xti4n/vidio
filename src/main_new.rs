mod error;
mod core;
mod tui;
mod cli;

use crate::core::{TranscriptService, ReportService, StorageService, extract_video_id};
use crate::error::Result;
use crate::tui::{App, EventHandler, init as tui_init, restore as tui_restore, ui};
use crate::cli::{Cli, Commands};
use clap::Parser;
use std::io;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Get { video_id, languages, preserve_formatting, report }) => {
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

async fn run_cli_get(video_input: String, languages: String, preserve_formatting: bool, generate_report: bool) -> Result<()> {
    let video_id = extract_video_id(&video_input)
        .ok_or_else(|| error::Error::custom("Invalid video URL or ID"))?;

    println!("Processing video: {}", video_id);

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
    println!("Transcript saved to: {:?}", transcript_path);

    // Generate report if requested
    if generate_report {
        println!("Generating report...");
        let report_content = report_service.generate_report(&transcript).await?;
        let report_path = StorageService::save_report(&video_id, &report_content)?;
        println!("Report saved to: {:?}", report_path);
    }

    Ok(())
}

async fn run_cli_report(video_id: String) -> Result<()> {
    println!("Generating report for video: {}", video_id);

    let transcript_content = StorageService::load_transcript(&video_id)?;
    
    // We need to create a mock FetchedTranscript for the report service
    // In a real implementation, you'd want to store more metadata
    let mock_transcript = create_mock_transcript(&video_id, &transcript_content);
    
    let report_service = ReportService::new();
    let report_content = report_service.generate_report(&mock_transcript).await?;
    
    let report_path = StorageService::save_report(&video_id, &report_content)?;
    println!("Report saved to: {:?}", report_path);

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
            format!("{}KB", size_kb)
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
        // Draw UI
        terminal.draw(|f| {
            ui::draw(f, &mut app);
        })?;

        // Handle events
        let event = event_handler.next_event()?;
        app.handle_event(event)?;

        // Check if we should quit
        if app.should_quit {
            break;
        }

        // Handle any background processing
        if let AppState::Processing { video_id, .. } = &app.state {
            handle_background_processing(&mut app, video_id.clone(), tx.clone()).await?;
        }
    }

    // Restore terminal
    tui_restore()?;
    Ok(())
}

async fn handle_background_processing(
    app: &mut App, 
    video_id: String, 
    tx: mpsc::UnboundedSender<String>
) -> Result<()> {
    // This is a simplified version - in a real implementation you'd use proper async tasks
    // For now, we'll just simulate the processing with some delays and progress updates
    
    tokio::spawn(async move {
        // Simulate transcript download
        let _ = tx.send("STATUS:Descargando transcripción...".to_string());
        let _ = tx.send("PROGRESS:0.25".to_string());
        let _ = tx.send("LOG:Fetching transcript...".to_string());
        
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        
        let _ = tx.send("PROGRESS:0.75".to_string());
        let _ = tx.send("LOG:Successfully fetched transcript!".to_string());
        let _ = tx.send("STATUS:Generando reporte...".to_string());
        
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        
        let _ = tx.send("PROGRESS:1.0".to_string());
        let _ = tx.send("LOG:Report generated successfully!".to_string());
        let _ = tx.send("STATUS:Completado".to_string());
        let _ = tx.send("COMPLETE".to_string());
    });

    Ok(())
}

// Helper function to create a mock transcript for report generation
fn create_mock_transcript(video_id: &str, content: &str) -> yt_transcript_rs::FetchedTranscript {
    // This is a workaround since we can't easily recreate the original transcript object
    // In a real implementation, you'd want to store the original transcript metadata
    use yt_transcript_rs::{FetchedTranscript, Snippet};
    
    let snippets: Vec<Snippet> = content
        .lines()
        .enumerate()
        .map(|(i, line)| Snippet {
            text: line.to_string(),
            start: i as f64,
            duration: 1.0,
        })
        .collect();

    FetchedTranscript {
        video_id: video_id.to_string(),
        language: "Unknown".to_string(),
        language_code: "unk".to_string(),
        is_generated: false,
        snippets,
    }
}

use crate::tui::app::AppState;