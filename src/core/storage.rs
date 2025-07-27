use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use yt_transcript_rs::FetchedTranscript;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub file_type: FileType,
    pub size: u64,
    pub modified: std::time::SystemTime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FileType {
    Transcript,
    Report,
}

pub struct StorageService;

impl StorageService {
    pub fn save_transcript(transcript: &FetchedTranscript) -> Result<PathBuf> {
        let file_name = format!("transcript_{}.txt", transcript.video_id);
        let path = PathBuf::from(&file_name);

        fs::write(&path, transcript.text())?;
        println!("Transcript saved to: {file_name}");

        Ok(path)
    }

    pub fn save_report(video_id: &str, content: &str) -> Result<PathBuf> {
        let file_name = format!("report_{video_id}.md");
        let path = PathBuf::from(&file_name);

        fs::write(&path, content)?;
        println!("Report saved to: {file_name}");

        Ok(path)
    }

    pub fn load_transcript(video_id: &str) -> Result<String> {
        let file_name = format!("transcript_{video_id}.txt");
        let content = fs::read_to_string(file_name)?;
        Ok(content)
    }

    #[allow(dead_code)]
    pub fn load_report(video_id: &str) -> Result<String> {
        let file_name = format!("report_{video_id}.md");
        let content = fs::read_to_string(file_name)?;
        Ok(content)
    }

    pub fn list_files() -> Result<Vec<FileEntry>> {
        let mut files = Vec::new();

        for entry in fs::read_dir(".")? {
            let entry = entry?;
            let path = entry.path();

            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                let file_type = if name.starts_with("transcript_") && name.ends_with(".txt") {
                    Some(FileType::Transcript)
                } else if name.starts_with("report_") && name.ends_with(".md") {
                    Some(FileType::Report)
                } else {
                    None
                };

                if let Some(file_type) = file_type {
                    let metadata = entry.metadata()?;
                    files.push(FileEntry {
                        path: path.clone(),
                        name: name.to_string(),
                        file_type,
                        size: metadata.len(),
                        modified: metadata.modified()?,
                    });
                }
            }
        }

        // Sort by modification time (newest first)
        files.sort_by(|a, b| b.modified.cmp(&a.modified));

        Ok(files)
    }

    pub fn delete_file(path: &Path) -> Result<()> {
        fs::remove_file(path)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn file_exists(file_name: &str) -> bool {
        Path::new(file_name).exists()
    }
}

impl FileEntry {
    #[allow(dead_code)]
    pub fn video_id(&self) -> Option<String> {
        let name = &self.name;
        if name.starts_with("transcript_") && name.ends_with(".txt") {
            Some(
                name.trim_start_matches("transcript_")
                    .trim_end_matches(".txt")
                    .to_string(),
            )
        } else if name.starts_with("report_") && name.ends_with(".md") {
            Some(
                name.trim_start_matches("report_")
                    .trim_end_matches(".md")
                    .to_string(),
            )
        } else {
            None
        }
    }
}
