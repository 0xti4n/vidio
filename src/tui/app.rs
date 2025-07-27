use crate::core::{FileType, ReportService, StorageService, TranscriptService, storage::FileEntry};
use crate::error::Result;
use crate::tui::components::{FileList, InputField, ProgressBar, Viewer};
use crate::tui::events::AppEvent;
use crossterm::event::{KeyCode, KeyEvent};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::sync::mpsc;

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Home,
    NewTranscript,
    Processing {
        video_id: String,
        progress: f64,
        status: String,
        logs: Vec<String>,
    },
    Browser {
        filter: FileFilter,
        search: String,
    },
    Viewer {
        file_path: PathBuf,
    },
    Settings,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FileFilter {
    All,
    Transcripts,
    Reports,
}

#[derive(Debug, Clone)]
pub struct TranscriptRequest {
    pub video_url: String,
    pub languages: Vec<String>,
    pub preserve_formatting: bool,
    pub generate_report: bool,
}

pub struct App {
    pub state: AppState,
    pub should_quit: bool,

    // Home screen
    pub selected_option: usize,

    // New transcript screen
    pub url_input: InputField,
    pub languages_input: InputField,
    pub preserve_formatting: bool,
    pub generate_report: bool,
    pub input_focus: usize,

    // Browser screen
    pub file_list: FileList,
    pub search_input: InputField,
    pub filter: FileFilter,

    // Viewer screen
    pub content_viewer: Option<Viewer>,
    pub viewer_height: u16,

    // Processing screen
    pub progress_bar: ProgressBar,

    // Services
    pub transcript_service: TranscriptService,
    pub report_service: ReportService,

    // Async communication
    pub processing_tx: Option<mpsc::UnboundedSender<String>>,
    pub processing_rx: Option<mpsc::UnboundedReceiver<String>>,
}

impl App {
    pub fn new() -> Result<Self> {
        let transcript_service = TranscriptService::new()?;
        let report_service = ReportService::new();
        let files = StorageService::list_files().unwrap_or_default();

        Ok(Self {
            state: AppState::Home,
            should_quit: false,

            selected_option: 0,

            url_input: InputField::new("Video URL", "https://youtu.be/..."),
            languages_input: InputField::new("Languages", "en,es"),
            preserve_formatting: true,
            generate_report: true,
            input_focus: 0,

            file_list: FileList::new(files),
            search_input: InputField::new("Search", "Filter files..."),
            filter: FileFilter::All,

            content_viewer: None,
            viewer_height: 0,
            progress_bar: ProgressBar::new(),

            transcript_service,
            report_service,

            processing_tx: None,
            processing_rx: None,
        })
    }

    pub fn handle_event(&mut self, event: AppEvent) -> Result<()> {
        match event {
            AppEvent::Quit => {
                self.should_quit = true;
            }
            AppEvent::Key(key) => {
                self.handle_key(key)?;
            }
            AppEvent::Tick => {
                // Handle any periodic updates
                self.handle_tick()?;
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        match &self.state {
            AppState::Home => self.handle_home_key(key),
            AppState::NewTranscript => self.handle_new_transcript_key(key),
            AppState::Browser { .. } => self.handle_browser_key(key),
            AppState::Viewer { .. } => self.handle_viewer_key(key),
            AppState::Processing { .. } => self.handle_processing_key(key),
            AppState::Settings => self.handle_settings_key(key),
        }
    }

    fn handle_home_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Up => {
                if self.selected_option > 0 {
                    self.selected_option -= 1;
                }
            }
            KeyCode::Down => {
                if self.selected_option < 3 {
                    self.selected_option += 1;
                }
            }
            KeyCode::Char('1') => self.selected_option = 0,
            KeyCode::Char('2') => self.selected_option = 1,
            KeyCode::Char('3') => self.selected_option = 2,
            KeyCode::Char('4') => self.selected_option = 3,
            KeyCode::Enter => match self.selected_option {
                0 => {
                    self.state = AppState::NewTranscript;
                    self.url_input.clear();
                    self.languages_input.value = "en,es".to_string();
                    self.url_input.focused = true;
                    self.input_focus = 0;
                }
                1 => {
                    self.refresh_file_list()?;
                    self.state = AppState::Browser {
                        filter: FileFilter::Transcripts,
                        search: String::new(),
                    };
                }
                2 => {
                    self.refresh_file_list()?;
                    self.state = AppState::Browser {
                        filter: FileFilter::Reports,
                        search: String::new(),
                    };
                }
                3 => {
                    self.state = AppState::Settings;
                }
                _ => {}
            },
            _ => {}
        }
        Ok(())
    }

    fn handle_new_transcript_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.state = AppState::Home;
            }
            KeyCode::Tab => {
                self.cycle_input_focus();
            }
            KeyCode::Enter => {
                if self.input_focus < 2 {
                    self.cycle_input_focus();
                } else {
                    self.start_processing()?;
                }
            }
            KeyCode::Char(' ') if self.input_focus == 2 => {
                self.preserve_formatting = !self.preserve_formatting;
            }
            KeyCode::Char(' ') if self.input_focus == 3 => {
                self.generate_report = !self.generate_report;
            }
            _ => {
                if self.input_focus == 0 {
                    self.url_input.handle_key(key);
                } else if self.input_focus == 1 {
                    self.languages_input.handle_key(key);
                }
            }
        }
        Ok(())
    }

    fn handle_browser_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.state = AppState::Home;
            }
            KeyCode::Enter => {
                if let Some(file) = self.file_list.get_selected() {
                    self.open_file(file.clone())?;
                }
            }
            KeyCode::Delete => {
                self.delete_selected_files()?;
            }
            KeyCode::Char('/') => {
                // Start search mode
                self.search_input.focused = true;
            }
            KeyCode::Char('1') => {
                self.filter = FileFilter::All;
                self.apply_filter();
            }
            KeyCode::Char('2') => {
                self.filter = FileFilter::Transcripts;
                self.apply_filter();
            }
            KeyCode::Char('3') => {
                self.filter = FileFilter::Reports;
                self.apply_filter();
            }
            _ => {
                if self.search_input.focused {
                    if key.code == KeyCode::Esc {
                        self.search_input.focused = false;
                        self.search_input.clear();
                        self.apply_filter();
                    } else {
                        self.search_input.handle_key(key);
                        self.apply_search_filter();
                    }
                } else {
                    self.file_list.handle_key(key);
                }
            }
        }
        Ok(())
    }

    fn handle_viewer_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.state = AppState::Browser {
                    filter: self.filter.clone(),
                    search: String::new(),
                };
            }
            _ => {
                if let Some(viewer) = &mut self.content_viewer {
                    viewer.handle_key(key, self.viewer_height); // Approximate height
                }
            }
        }
        Ok(())
    }

    fn handle_processing_key(&mut self, key: KeyEvent) -> Result<()> {
        if key.code == KeyCode::Esc {
            // Cancel processing
            self.state = AppState::NewTranscript;
            self.progress_bar.reset();
        }
        Ok(())
    }

    fn handle_settings_key(&mut self, key: KeyEvent) -> Result<()> {
        if key.code == KeyCode::Esc {
            self.state = AppState::Home;
        }
        Ok(())
    }

    fn handle_tick(&mut self) -> Result<()> {
        // Handle any async messages
        let mut messages = Vec::new();
        if let Some(rx) = &mut self.processing_rx {
            while let Ok(message) = rx.try_recv() {
                messages.push(message);
            }
        }

        for message in messages {
            if message.starts_with("PROGRESS:") {
                if let Ok(progress) = message.trim_start_matches("PROGRESS:").parse::<f64>() {
                    self.progress_bar.set_progress(progress);
                }
            } else if message.starts_with("STATUS:") {
                let status = message.trim_start_matches("STATUS:").to_string();
                self.progress_bar.set_message(status);
            } else if message.starts_with("LOG:") {
                let log = message.trim_start_matches("LOG:").to_string();
                self.progress_bar.add_log(log);
            } else if message == "COMPLETE" {
                self.refresh_file_list()?;
                self.state = AppState::Home;
                self.progress_bar.reset();
            }
        }
        Ok(())
    }

    fn cycle_input_focus(&mut self) {
        self.url_input.focused = false;
        self.languages_input.focused = false;

        self.input_focus = (self.input_focus + 1) % 4;

        match self.input_focus {
            0 => self.url_input.focused = true,
            1 => self.languages_input.focused = true,
            _ => {}
        }
    }

    fn start_processing(&mut self) -> Result<()> {
        if !self.url_input.is_valid() {
            return Ok(());
        }

        let request = TranscriptRequest {
            video_url: self.url_input.value.clone(),
            languages: self
                .languages_input
                .value
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            preserve_formatting: self.preserve_formatting,
            generate_report: self.generate_report,
        };

        if let Some(video_id) = crate::core::transcript::extract_video_id(&request.video_url) {
            self.state = AppState::Processing {
                video_id: video_id.clone(),
                progress: 0.0,
                status: "Starting...".to_string(),
                logs: Vec::new(),
            };

            self.progress_bar.reset();
            self.progress_bar.set_message("Starting...".to_string());

            // Start real async processing
            if let Some(tx) = &self.processing_tx {
                self.start_real_processing(video_id, request, tx.clone());
            }
        }

        Ok(())
    }

    fn start_real_processing(
        &self,
        video_id: String,
        request: TranscriptRequest,
        tx: mpsc::UnboundedSender<String>,
    ) {
        // Clone the services for the async task
        let transcript_service = self.transcript_service.clone();
        let report_service = self.report_service.clone();

        tokio::spawn(async move {
            let _ = tx.send("STATUS:Starting processing...".to_string());
            let _ = tx.send("PROGRESS:0.1".to_string());
            let _ = tx.send("LOG:Extracting video ID...".to_string());

            // Convert languages to the correct format
            let languages: Vec<&str> = request.languages.iter().map(|s| s.as_str()).collect();

            // Fetch transcript
            let _ = tx.send("STATUS:Downloading transcript...".to_string());
            let _ = tx.send("PROGRESS:0.25".to_string());
            let _ = tx.send("LOG:Fetching transcript...".to_string());

            match transcript_service
                .fetch_transcript(&video_id, &languages, request.preserve_formatting)
                .await
            {
                Ok(transcript) => {
                    let _ = tx.send("PROGRESS:0.5".to_string());
                    let _ = tx.send("LOG:Successfully fetched transcript!".to_string());
                    let _ = tx.send("LOG:Saving transcript to file...".to_string());

                    // Save transcript
                    match StorageService::save_transcript(&transcript) {
                        Ok(_) => {
                            let _ = tx.send("PROGRESS:0.6".to_string());
                            let _ = tx.send("LOG:Transcript saved successfully!".to_string());

                            // Generate report if requested
                            if request.generate_report {
                                let _ = tx.send("STATUS:Generating report...".to_string());
                                let _ = tx.send("PROGRESS:0.7".to_string());
                                let _ = tx.send("LOG:Generating report...".to_string());

                                match report_service.generate_report(&transcript).await {
                                    Ok(report_content) => {
                                        let _ = tx.send("PROGRESS:0.9".to_string());
                                        let _ = tx
                                            .send("LOG:Report generated successfully!".to_string());
                                        let _ = tx.send("LOG:Saving report to file...".to_string());

                                        match StorageService::save_report(
                                            &video_id,
                                            &report_content,
                                        ) {
                                            Ok(_) => {
                                                let _ = tx.send("PROGRESS:1.0".to_string());
                                                let _ = tx.send(
                                                    "LOG:Report saved successfully!".to_string(),
                                                );
                                                let _ = tx.send("STATUS:Completed".to_string());
                                                let _ = tx.send("COMPLETE".to_string());
                                            }
                                            Err(e) => {
                                                let _ = tx
                                                    .send(format!("LOG:Error saving report: {e}"));
                                                let _ = tx
                                                    .send("STATUS:Error saving report".to_string());
                                                let _ = tx.send("COMPLETE".to_string());
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        let _ =
                                            tx.send(format!("LOG:Error generating report: {e}"));
                                        let _ =
                                            tx.send("STATUS:Error generating report".to_string());
                                        let _ = tx.send("COMPLETE".to_string());
                                    }
                                }
                            } else {
                                let _ = tx.send("PROGRESS:1.0".to_string());
                                let _ = tx.send("STATUS:Completed".to_string());
                                let _ = tx.send("COMPLETE".to_string());
                            }
                        }
                        Err(e) => {
                            let _ = tx.send(format!("LOG:Error saving transcript: {e}"));
                            let _ = tx.send("STATUS:Error saving transcript".to_string());
                            let _ = tx.send("COMPLETE".to_string());
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(format!("LOG:Error fetching transcript: {e}"));
                    let _ = tx.send("STATUS:Error downloading transcript".to_string());
                    let _ = tx.send("COMPLETE".to_string());
                }
            }
        });
    }

    fn refresh_file_list(&mut self) -> Result<()> {
        let files = StorageService::list_files()?;
        self.file_list.update_items(files);
        Ok(())
    }

    fn apply_filter(&mut self) {
        let all_files = StorageService::list_files().unwrap_or_default();
        let filtered_files: Vec<FileEntry> = all_files
            .into_iter()
            .filter(|file| match self.filter {
                FileFilter::All => true,
                FileFilter::Transcripts => file.file_type == FileType::Transcript,
                FileFilter::Reports => file.file_type == FileType::Report,
            })
            .collect();

        self.file_list.update_items(filtered_files);
    }

    fn apply_search_filter(&mut self) {
        let search_term = self.search_input.value.to_lowercase();
        if search_term.is_empty() {
            self.apply_filter();
            return;
        }

        let all_files = StorageService::list_files().unwrap_or_default();
        let filtered_files: Vec<FileEntry> = all_files
            .into_iter()
            .filter(|file| {
                let matches_filter = match self.filter {
                    FileFilter::All => true,
                    FileFilter::Transcripts => file.file_type == FileType::Transcript,
                    FileFilter::Reports => file.file_type == FileType::Report,
                };

                let matches_search = file.name.to_lowercase().contains(&search_term);

                matches_filter && matches_search
            })
            .collect();

        self.file_list.update_items(filtered_files);
    }

    fn open_file(&mut self, file: FileEntry) -> Result<()> {
        let content = std::fs::read_to_string(&file.path)?;
        let viewer = Viewer::new(content, file.path.to_string_lossy().to_string());
        self.content_viewer = Some(viewer);
        self.state = AppState::Viewer {
            file_path: file.path,
        };
        Ok(())
    }

    fn delete_selected_files(&mut self) -> Result<()> {
        let selected_files = self.file_list.get_selected_items();
        for file in selected_files {
            StorageService::delete_file(&file.path)?;
        }
        self.refresh_file_list()?;
        Ok(())
    }
}
