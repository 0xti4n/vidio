use crate::error::Result;
use yt_transcript_rs::{FetchedTranscript, api::YouTubeTranscriptApi};

#[derive(Clone)]
pub struct TranscriptService {
    api: YouTubeTranscriptApi,
}

impl TranscriptService {
    pub fn new() -> Result<Self> {
        let api = YouTubeTranscriptApi::new(None, None, None)?;
        Ok(Self { api })
    }

    pub async fn fetch_transcript(
        &self,
        video_id: &str,
        languages: &[&str],
        preserve_formatting: bool,
    ) -> Result<FetchedTranscript> {
        // println!("Fetching transcript for video ID: {}", video_id);

        match self
            .api
            .fetch_transcript(video_id, languages, preserve_formatting)
            .await
        {
            Ok(transcript) => Ok(transcript),
            Err(e) => Err(crate::error::Error::custom(format!(
                "Failed to fetch transcript: {e}"
            ))),
        }
    }

    pub fn format_transcript(transcript: &FetchedTranscript) -> Vec<String> {
        transcript
            .snippets
            .iter()
            .map(|snippet| {
                let start = format_timestamp(snippet.start);
                let end = format_timestamp(snippet.start + snippet.duration);
                format!("[{start} - {end}] {}", snippet.text.trim())
            })
            .collect()
    }
}

fn format_timestamp(seconds: f64) -> String {
    let total_millis = (seconds * 1000.0).round() as u64;
    let hours = total_millis / 3_600_000;
    let minutes = (total_millis % 3_600_000) / 60_000;
    let secs = (total_millis % 60_000) / 1_000;
    let millis = total_millis % 1_000;

    if hours > 0 {
        format!("{hours:02}:{minutes:02}:{secs:02}.{millis:03}")
    } else {
        format!("{minutes:02}:{secs:02}.{millis:03}")
    }
}

pub fn extract_video_id(url: &str) -> Option<String> {
    // Extract video ID from various YouTube URL formats
    if let Some(v_param) = url.split("v=").nth(1) {
        Some(v_param.split('&').next().unwrap_or(v_param).to_string())
    } else if let Some(youtu_be) = url.split("youtu.be/").nth(1) {
        Some(youtu_be.split('?').next().unwrap_or(youtu_be).to_string())
    } else if url.len() == 11
        && url
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        Some(url.to_string())
    } else {
        None
    }
}
