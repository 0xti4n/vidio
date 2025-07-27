use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph},
};

pub struct ProgressBar {
    pub progress: f64,
    pub message: String,
    pub logs: Vec<String>,
    pub max_logs: usize,
}

impl ProgressBar {
    pub fn new() -> Self {
        Self {
            progress: 0.0,
            message: String::new(),
            logs: Vec::new(),
            max_logs: 10,
        }
    }

    pub fn set_progress(&mut self, progress: f64) {
        self.progress = progress.clamp(0.0, 1.0);
    }

    pub fn set_message(&mut self, message: String) {
        self.message = message;
    }

    pub fn add_log(&mut self, log: String) {
        let timestamp = chrono::Local::now().format("%H:%M:%S");
        let log_entry = format!("[{timestamp}] {log}");

        self.logs.push(log_entry);

        if self.logs.len() > self.max_logs {
            self.logs.remove(0);
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect, video_id: &str) {
        let chunks = ratatui::layout::Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints([
                ratatui::layout::Constraint::Length(3), // Video ID
                ratatui::layout::Constraint::Length(3), // Progress bar
                ratatui::layout::Constraint::Length(3), // Status
                ratatui::layout::Constraint::Min(1),    // Logs
            ])
            .split(area);

        // Video ID
        let video_paragraph = Paragraph::new(format!("Video ID: {video_id}"))
            .style(Style::default().fg(Color::White));
        f.render_widget(video_paragraph, chunks[0]);

        // Progress bar
        let progress_percent = (self.progress * 100.0) as u16;
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Progreso"))
            .gauge_style(Style::default().fg(Color::Green))
            .percent(progress_percent);
        f.render_widget(gauge, chunks[1]);

        // Status message
        let status_paragraph = Paragraph::new(format!("Estado: {}", self.message))
            .style(Style::default().fg(Color::Yellow));
        f.render_widget(status_paragraph, chunks[2]);

        // Logs
        let log_lines: Vec<Line> = self
            .logs
            .iter()
            .map(|log| Line::from(Span::raw(log)))
            .collect();

        let logs_paragraph =
            Paragraph::new(log_lines).block(Block::default().borders(Borders::ALL).title("Log"));
        f.render_widget(logs_paragraph, chunks[3]);
    }

    pub fn reset(&mut self) {
        self.progress = 0.0;
        self.message.clear();
        self.logs.clear();
    }
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new()
    }
}
