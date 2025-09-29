use crossterm::event::{self, Event, KeyEvent, MouseEvent};
use std::time::Duration;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum AppEvent {
    Quit,
    Key(KeyEvent),
    Mouse(MouseEvent),
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
                Event::Key(key) => Ok(AppEvent::Key(key)),
                Event::Mouse(mouse) => Ok(AppEvent::Mouse(mouse)),
                Event::Resize(_, _) => Ok(AppEvent::Tick),
                _ => Ok(AppEvent::Tick),
            }
        } else {
            Ok(AppEvent::Tick)
        }
    }
}
