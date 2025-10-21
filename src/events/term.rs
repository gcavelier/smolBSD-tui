use ratatui::crossterm;
use std::sync::mpsc::Sender;

use crate::events::AppEvent;

/// This function listens for events from the terminal and send the relevant ones as `AppEvent`
pub fn get_term_events(tx: Sender<AppEvent>) {
    loop {
        let event = crossterm::event::read().unwrap();
        match event {
            crossterm::event::Event::FocusGained => (),
            crossterm::event::Event::FocusLost => (),
            crossterm::event::Event::Key(key_event) => tx.send(AppEvent::Key(key_event)).unwrap(),
            crossterm::event::Event::Mouse(_) => (),
            crossterm::event::Event::Paste(_) => (),
            crossterm::event::Event::Resize(_, _) => tx.send(AppEvent::ForceRender).unwrap(),
        }
    }
}
