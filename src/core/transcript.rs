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
            Ok(transcript) => {
                // println!("Successfully fetched transcript!");
                // println!("Video ID: {}", transcript.video_id);
                // println!(
                //     "Language: {} ({})",
                //     transcript.language, transcript.language_code
                // );
                // println!("Is auto-generated: {}", transcript.is_generated);
                // println!("Number of snippets: {}", transcript.snippets.len());

                Ok(transcript)
            }
            Err(e) => Err(crate::error::Error::custom(format!(
                "Failed to fetch transcript: {e}"
            ))),
        }
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
