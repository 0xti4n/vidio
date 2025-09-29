use crate::error::{Error, Result};
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
    let raw_id = if let Some(v_param) = url.split("v=").nth(1) {
        v_param.split('&').next().unwrap_or(v_param)
    } else if let Some(youtu_be) = url.split("youtu.be/").nth(1) {
        youtu_be.split('?').next().unwrap_or(youtu_be)
    } else {
        url
    };

    sanitize_video_id(raw_id).ok()
}

const MAX_VIDEO_ID_LEN: usize = 128;

/// Ensure a video identifier is safe for downstream use (filesystem paths, API calls, etc.).
/// Only ASCII alphanumeric characters plus `_` and `-` are allowed.
pub fn sanitize_video_id(raw: &str) -> Result<String> {
    let trimmed = raw.trim();

    if trimmed.is_empty() {
        return Err(Error::custom("Video ID cannot be empty"));
    }

    if trimmed.len() > MAX_VIDEO_ID_LEN {
        return Err(Error::custom("Video ID is unexpectedly long"));
    }

    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_'))
    {
        return Err(Error::custom(
            "Video ID contains unsupported characters; expected only letters, numbers, '-' or '_'",
        ));
    }

    Ok(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::{MAX_VIDEO_ID_LEN, sanitize_video_id};

    #[test]
    fn allows_expected_characters() {
        let id = sanitize_video_id("abcDEF123-_x").expect("valid ID");
        assert_eq!(id, "abcDEF123-_x");
    }

    #[test]
    fn rejects_empty() {
        assert!(sanitize_video_id("   ").is_err());
    }

    #[test]
    fn rejects_invalid_chars() {
        assert!(sanitize_video_id("abc/../../etc").is_err());
    }

    #[test]
    fn rejects_too_long() {
        let long = "a".repeat(MAX_VIDEO_ID_LEN + 1);
        assert!(sanitize_video_id(&long).is_err());
    }
}
