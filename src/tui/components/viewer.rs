use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub struct ContentViewer {
    pub content: String,
    pub scroll: usize,
    pub file_path: String,
}

impl ContentViewer {
    pub fn new(content: String, file_path: String) -> Self {
        Self {
            content,
            scroll: 0,
            file_path,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent, area_height: usize) -> bool {
        match key.code {
            KeyCode::Up => {
                if self.scroll > 0 {
                    self.scroll -= 1;
                }
                true
            }
            KeyCode::Down => {
                let lines = self.content.lines().count();
                if self.scroll < lines.saturating_sub(area_height.saturating_sub(2)) {
                    self.scroll += 1;
                }
                true
            }
            KeyCode::PageUp => {
                self.scroll = self.scroll.saturating_sub(area_height.saturating_sub(2));
                true
            }
            KeyCode::PageDown => {
                let lines = self.content.lines().count();
                let page_size = area_height.saturating_sub(2);
                self.scroll = (self.scroll + page_size).min(lines.saturating_sub(page_size));
                true
            }
            KeyCode::Home => {
                self.scroll = 0;
                true
            }
            KeyCode::End => {
                let lines = self.content.lines().count();
                let page_size = area_height.saturating_sub(2);
                self.scroll = lines.saturating_sub(page_size);
                true
            }
            _ => false,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let title = format!(
            "Visor: {}",
            std::path::Path::new(&self.file_path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
        );

        let lines: Vec<Line> = self
            .content
            .lines()
            .skip(self.scroll)
            .take(area.height.saturating_sub(2) as usize)
            .map(|line| {
                if line.starts_with('#') {
                    // Markdown headers
                    Line::from(Span::styled(line, Style::default().fg(Color::Yellow)))
                } else if line.starts_with('|') && line.ends_with('|') {
                    // Table rows
                    Line::from(Span::styled(line, Style::default().fg(Color::Cyan)))
                } else if line.starts_with('-') || line.starts_with('*') {
                    // List items
                    Line::from(Span::styled(line, Style::default().fg(Color::Green)))
                } else {
                    // Regular text
                    Line::from(Span::raw(line))
                }
            })
            .collect();

        let total_lines = self.content.lines().count();
        let visible_lines = area.height.saturating_sub(2) as usize;
        let scroll_info = if total_lines > visible_lines {
            format!(
                " (Línea {}-{} de {})",
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
        self.file_path = file_path;
        self.scroll = 0;
    }
}
