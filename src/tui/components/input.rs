use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

#[derive(Debug, Clone)]
pub struct InputField {
    pub value: String,
    pub cursor: usize,
    pub placeholder: String,
    pub label: String,
    pub focused: bool,
}

impl InputField {
    pub fn new(label: &str, placeholder: &str) -> Self {
        Self {
            value: String::new(),
            cursor: 0,
            placeholder: placeholder.to_string(),
            label: label.to_string(),
            focused: false,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char(c) => {
                self.value.insert(self.cursor, c);
                self.cursor += 1;
                true
            }
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.value.remove(self.cursor);
                }
                true
            }
            KeyCode::Delete => {
                if self.cursor < self.value.len() {
                    self.value.remove(self.cursor);
                }
                true
            }
            KeyCode::Left => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
                true
            }
            KeyCode::Right => {
                if self.cursor < self.value.len() {
                    self.cursor += 1;
                }
                true
            }
            KeyCode::Home => {
                self.cursor = 0;
                true
            }
            KeyCode::End => {
                self.cursor = self.value.len();
                true
            }
            _ => false,
        }
    }

    pub fn render(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(self.label.as_str())
            .border_style(if self.focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Gray)
            });

        let text = if self.value.is_empty() && !self.focused {
            Line::from(Span::styled(
                &self.placeholder,
                Style::default().fg(Color::DarkGray),
            ))
        } else {
            let mut spans = vec![];

            if self.focused && self.cursor <= self.value.len() {
                let (before, after) = self.value.split_at(self.cursor);
                spans.push(Span::raw(before));
                spans.push(Span::styled("│", Style::default().fg(Color::Yellow)));
                spans.push(Span::raw(after));
            } else {
                spans.push(Span::raw(&self.value));
            }

            Line::from(spans)
        };

        let paragraph = Paragraph::new(text).block(block);
        f.render_widget(paragraph, area);
    }

    pub fn is_valid(&self) -> bool {
        !self.value.trim().is_empty()
    }

    pub fn clear(&mut self) {
        self.value.clear();
        self.cursor = 0;
    }
}
