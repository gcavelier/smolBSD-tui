mod app;
mod events;
mod ui;
mod vm;

use crate::events::{AppEvent, get_fs_events, get_term_events};
use std::sync::mpsc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel::<AppEvent>();

    let mut app = app::State::new(tx.clone())?;

    // Start a thread to listen to crossterm events
    let tx_clone = tx.clone();
    let term_thread = std::thread::spawn(|| get_term_events(tx));

    // Start a thread to handle FS events
    let base_dir_clone = app.base_dir.clone();
    let fs_notify_thread = std::thread::spawn(|| get_fs_events(tx_clone, base_dir_clone));

    ratatui::run(|terminal| {
        while !app.exit {
            terminal.draw(|frame| ui::render(frame, &mut app)).unwrap();

            // Wait for an event
            let event = rx.recv().unwrap();

            if let Err(err) = events::handle(&mut app, event) {
                app.fatal_error
                    .get_or_insert(format!("failed to handle event : {err}"));
                break;
            }

            if term_thread.is_finished() {
                app.fatal_error
                    .get_or_insert("term_thread has finished!".to_owned());
                break;
            }

            if fs_notify_thread.is_finished() {
                app.fatal_error
                    .get_or_insert("fs_notify_thread has finished!".to_owned());
                break;
            }
        }
    });

    if let Some(err) = app.fatal_error {
        Err(err.into())
    } else {
        Ok(())
    }
}
