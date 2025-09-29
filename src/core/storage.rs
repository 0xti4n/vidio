use crate::core::transcript;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::fs as std_fs;
use std::path::{Path, PathBuf};
use yt_transcript_rs::FetchedTranscript;

use tokio::fs;

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
    fn ensure_directories() -> Result<()> {
        std_fs::create_dir_all("transcripts")?;
        std_fs::create_dir_all("reports")?;
        Ok(())
    }

    fn transcript_path(video_id: &str) -> PathBuf {
        PathBuf::from("transcripts").join(format!("transcript_{video_id}.txt"))
    }

    fn report_path(video_id: &str) -> PathBuf {
        PathBuf::from("reports").join(format!("report_{video_id}.md"))
    }

    pub fn transcript_exists(video_id: &str) -> bool {
        if Self::ensure_directories().is_err() {
            return false;
        }
        Self::transcript_path(video_id).exists()
    }

    pub fn report_exists(video_id: &str) -> bool {
        if Self::ensure_directories().is_err() {
            return false;
        }
        Self::report_path(video_id).exists()
    }

    pub async fn save_transcript(transcript: &FetchedTranscript) -> Result<PathBuf> {
        Self::ensure_directories()?;
        let path = Self::transcript_path(&transcript.video_id);

        let formatted_transcript = transcript::TranscriptService::format_transcript(transcript);
        let content = formatted_transcript.join("\n");
        fs::write(&path, &content).await?;
        println!("Transcript saved to: {}", path.display());

        Ok(path)
    }

    pub async fn save_report(video_id: &str, content: &str) -> Result<PathBuf> {
        Self::ensure_directories()?;

        let path = Self::report_path(video_id);

        fs::write(&path, content).await?;
        println!("Report saved to: {}", path.display());

        Ok(path)
    }

    pub async fn load_transcript(video_id: &str) -> Result<String> {
        let path = Self::transcript_path(video_id);
        let content = fs::read_to_string(path).await?;
        Ok(content)
    }

    #[allow(dead_code)]
    pub async fn load_report(video_id: &str) -> Result<String> {
        let path = Self::report_path(video_id);
        let content = fs::read_to_string(path).await?;
        Ok(content)
    }

    pub fn list_files() -> Result<Vec<FileEntry>> {
        Self::ensure_directories()?;
        let mut files = Vec::new();

        // Check transcripts folder
        if let Ok(entries) = std_fs::read_dir("transcripts") {
            for entry in entries {
                let entry = entry?;
                let path = entry.path();

                if let Some(name) = path.file_name().and_then(|n| n.to_str())
                    && name.starts_with("transcript_")
                    && name.ends_with(".txt")
                {
                    let metadata = entry.metadata()?;
                    files.push(FileEntry {
                        path: path.clone(),
                        name: name.to_string(),
                        file_type: FileType::Transcript,
                        size: metadata.len(),
                        modified: metadata.modified()?,
                    });
                }
            }
        }

        // Check reports folder
        if let Ok(entries) = std_fs::read_dir("reports") {
            for entry in entries {
                let entry = entry?;
                let path = entry.path();

                if let Some(name) = path.file_name().and_then(|n| n.to_str())
                    && name.starts_with("report_")
                    && name.ends_with(".md")
                {
                    let metadata = entry.metadata()?;
                    files.push(FileEntry {
                        path: path.clone(),
                        name: name.to_string(),
                        file_type: FileType::Report,
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
        std_fs::remove_file(path)?;
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
