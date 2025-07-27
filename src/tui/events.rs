use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum AppEvent {
    Quit,
    Key(KeyEvent),
    Tick,
}

pub struct EventHandler;

impl EventHandler {
    pub fn new() -> Self {
        Self
    }

    pub fn next_event(&self) -> crate::error::Result<AppEvent> {
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') => Ok(AppEvent::Quit),
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        Ok(AppEvent::Quit)
                    }
                    _ => Ok(AppEvent::Key(key)),
                },
                _ => Ok(AppEvent::Tick),
            }
        } else {
            Ok(AppEvent::Tick)
        }
    }
}
