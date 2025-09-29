use crate::core::storage::FileEntry;
use crossterm::event::{KeyCode, KeyEvent, MouseEvent, MouseEventKind};
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
    viewport_size: usize,
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
            viewport_size: 0,
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
            KeyCode::PageDown => {
                self.page_down();
                true
            }
            KeyCode::PageUp => {
                self.page_up();
                true
            }
            KeyCode::Home => {
                self.go_home();
                true
            }
            KeyCode::End => {
                self.go_end();
                true
            }
            KeyCode::Char(' ') => {
                self.toggle_selected();
                true
            }
            _ => false,
        }
    }

    pub fn handle_mouse(&mut self, mouse: MouseEvent) -> bool {
        match mouse.kind {
            MouseEventKind::ScrollUp => {
                self.scroll_up();
                true
            }
            MouseEventKind::ScrollDown => {
                self.scroll_down();
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
        self.adjust_offset();
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
        self.adjust_offset();
    }

    fn page_down(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let step = self.viewport_size.max(1);
        let current = self.state.selected().unwrap_or(0);
        let new_index = (current + step).min(self.items.len() - 1);
        self.state.select(Some(new_index));
        self.adjust_offset();
    }

    fn page_up(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let step = self.viewport_size.max(1);
        let current = self.state.selected().unwrap_or(0);
        let new_index = current.saturating_sub(step);
        self.state.select(Some(new_index));
        self.adjust_offset();
    }

    fn go_home(&mut self) {
        if self.items.is_empty() {
            return;
        }
        self.state.select(Some(0));
        self.adjust_offset();
    }

    fn go_end(&mut self) {
        if self.items.is_empty() {
            return;
        }
        self.state.select(Some(self.items.len() - 1));
        self.adjust_offset();
    }

    fn scroll_up(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let current = self.state.selected().unwrap_or(0);
        if current == 0 {
            self.state.select(Some(0));
        } else {
            self.state.select(Some(current - 1));
        }
        self.adjust_offset();
    }

    fn scroll_down(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let current = self.state.selected().unwrap_or(0);
        let last = self.items.len() - 1;
        let new_index = (current + 1).min(last);
        self.state.select(Some(new_index));
        self.adjust_offset();
    }

    pub fn toggle_selected(&mut self) {
        if let Some(i) = self.state.selected()
            && i < self.selected_items.len()
        {
            self.selected_items[i] = !self.selected_items[i];
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
        self.viewport_size = area.height.saturating_sub(2) as usize;
        if self.viewport_size == 0 {
            self.viewport_size = 1;
        }
        self.adjust_offset();

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

        self.adjust_offset();
    }

    fn adjust_offset(&mut self) {
        if self.items.is_empty() {
            *self.state.offset_mut() = 0;
            return;
        }

        let viewport = self.viewport_size.max(1);
        let max_index = self.items.len() - 1;
        let selected = self
            .state
            .selected()
            .map(|idx| idx.min(max_index))
            .unwrap_or(0);
        self.state.select(Some(selected));

        let max_offset = self.items.len().saturating_sub(viewport);
        let offset = self.state.offset().min(max_offset);
        *self.state.offset_mut() = offset;

        if selected < offset {
            *self.state.offset_mut() = selected;
        } else if selected >= offset + viewport {
            *self.state.offset_mut() = selected + 1 - viewport;
        }
    }
}
