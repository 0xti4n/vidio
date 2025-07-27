use crate::core::storage::FileEntry;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState},
};

pub struct FileList {
    pub items: Vec<FileEntry>,
    pub state: ListState,
    pub selected_items: Vec<bool>,
}

impl FileList {
    pub fn new(items: Vec<FileEntry>) -> Self {
        let selected_items = vec![false; items.len()];
        let mut state = ListState::default();
        if !items.is_empty() {
            state.select(Some(0));
        }

        Self {
            items,
            state,
            selected_items,
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Up => {
                self.previous();
                true
            }
            KeyCode::Down => {
                self.next();
                true
            }
            KeyCode::Char(' ') => {
                self.toggle_selected();
                true
            }
            _ => false,
        }
    }

    pub fn next(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => (i + 1) % self.items.len(),
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn toggle_selected(&mut self) {
        if let Some(i) = self.state.selected() {
            if i < self.selected_items.len() {
                self.selected_items[i] = !self.selected_items[i];
            }
        }
    }

    pub fn get_selected(&self) -> Option<&FileEntry> {
        self.state.selected().and_then(|i| self.items.get(i))
    }

    pub fn get_selected_items(&self) -> Vec<&FileEntry> {
        self.selected_items
            .iter()
            .enumerate()
            .filter_map(
                |(i, &selected)| {
                    if selected { self.items.get(i) } else { None }
                },
            )
            .collect()
    }

    pub fn render(&mut self, f: &mut Frame, area: Rect, title: &str) {
        let items: Vec<ListItem> = self
            .items
            .iter()
            .enumerate()
            .map(|(i, file)| {
                let checkbox = if self.selected_items.get(i).copied().unwrap_or(false) {
                    "☑ "
                } else {
                    "☐ "
                };

                let icon = match file.file_type {
                    crate::core::storage::FileType::Transcript => "📄",
                    crate::core::storage::FileType::Report => "📊",
                };

                let size_kb = file.size / 1024;
                let size_str = if size_kb < 1024 {
                    format!("{size_kb}KB")
                } else {
                    format!("{:.1}MB", size_kb as f64 / 1024.0)
                };

                let line = Line::from(vec![
                    Span::raw(checkbox),
                    Span::raw(icon),
                    Span::raw(" "),
                    Span::styled(&file.name, Style::default().fg(Color::White)),
                    Span::raw(format!(" ({size_str})")),
                ]);

                ListItem::new(line)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title(title))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_stateful_widget(list, area, &mut self.state);
    }

    pub fn update_items(&mut self, new_items: Vec<FileEntry>) {
        let current_selected = self.state.selected();
        self.items = new_items;
        self.selected_items = vec![false; self.items.len()];

        if self.items.is_empty() {
            self.state.select(None);
        } else if let Some(selected) = current_selected {
            if selected >= self.items.len() {
                self.state.select(Some(self.items.len() - 1));
            }
        } else {
            self.state.select(Some(0));
        }
    }
}
