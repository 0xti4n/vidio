use crossterm::event::{KeyCode, KeyEvent};
use html_escape::decode_html_entities;
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use textwrap::wrap;

#[derive(Debug, Clone, Default)]
pub struct Viewer {
    pub content: String,
    pub title: String,
    pub scroll: usize,
    wrapped_lines: Vec<String>,
    last_known_width: u16,
}

impl Viewer {
    pub fn new(content: String, title: String) -> Self {
        Self {
            content,
            title,
            scroll: 0,
            wrapped_lines: Vec::new(),
            last_known_width: 0,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent, area_height: u16) {
        let area_height = area_height as usize;
        let lines = self.wrapped_lines.len();
        let page_size = area_height.saturating_sub(2);

        match key.code {
            KeyCode::Up => {
                if self.scroll > 0 {
                    self.scroll -= 1;
                }
            }
            KeyCode::Down => {
                if self.scroll < lines.saturating_sub(page_size) {
                    self.scroll += 1;
                }
            }
            KeyCode::PageUp => {
                self.scroll = self.scroll.saturating_sub(page_size);
            }
            KeyCode::PageDown => {
                self.scroll = (self.scroll + page_size).min(lines.saturating_sub(page_size));
            }
            KeyCode::Home => {
                self.scroll = 0;
            }
            KeyCode::End => {
                self.scroll = lines.saturating_sub(page_size);
            }
            _ => {}
        }
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect) {
        let view_width = area.width.saturating_sub(2) as usize;

        if area.width != self.last_known_width {
            let decoded_content = decode_html_entities(&self.content);
            let wrapped_text = wrap(&decoded_content, view_width);
            self.wrapped_lines = wrapped_text.into_iter().map(|s| s.to_string()).collect();
            self.last_known_width = area.width;
        }
        let title = format!(
            "Viewer: {}",
            std::path::Path::new(&self.title)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
        );

        let lines: Vec<Line> = self
            .wrapped_lines
            .iter()
            .skip(self.scroll)
            .take(area.height.saturating_sub(2) as usize)
            .map(|line| {
                let line_str = line.to_string();
                if line_str.starts_with('#') {
                    // Markdown headers
                    Line::from(Span::styled(line_str, Style::default().fg(Color::Yellow)))
                } else if line_str.starts_with('|') && line_str.ends_with('|') {
                    // Table rows
                    Line::from(Span::styled(line_str, Style::default().fg(Color::Cyan)))
                } else if line_str.starts_with('-') || line_str.starts_with('*') {
                    // List items
                    Line::from(Span::styled(line_str, Style::default().fg(Color::Green)))
                } else {
                    // Regular text
                    Line::from(Span::raw(line_str))
                }
            })
            .collect();

        let total_lines = self.wrapped_lines.len();
        let visible_lines = area.height.saturating_sub(2) as usize;
        let scroll_info = if total_lines > visible_lines {
            format!(
                " (Line {}-{} of {})",
                self.scroll + 1,
                (self.scroll + visible_lines).min(total_lines),
                total_lines
            )
        } else {
            String::new()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!("{title}{scroll_info}"));

        let paragraph = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false });

        f.render_widget(paragraph, area);
    }

    #[allow(dead_code)]
    pub fn set_content(&mut self, content: String, file_path: String) {
        self.content = content;
        self.title = file_path;
        self.scroll = 0;
        self.wrapped_lines = Vec::new();
        self.last_known_width = 0;
    }
}
