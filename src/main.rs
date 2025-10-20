use std::{path::Path, sync::mpsc::Receiver};

use ratatui::{DefaultTerminal, crossterm};

use crate::events::AppEvent;

mod app;
mod events;
mod ui;

fn get_base_dir_arg() -> Option<String> {
    let mut args_iter = std::env::args();
    if args_iter.len() != 2 {
        println!("\nThis program needs one argument, the path to a directory containing :");
        println!(" - the startnb.sh script");
        println!(" - the VMs configurations (etc/)");
        println!(" - the VMs kernels (kernels/) (Optionnal)");
        println!(" - the VMs images (images/) (Optionnal)\n");
        return None;
    }

    // We only care about the first argument
    // The first element in args_iter is the program name,
    // so we get the second element in args_iter
    if let Some(base_dir) = args_iter.nth(1) {
        if !Path::new(&base_dir).is_dir() {
            println!("'{base_dir}' is not a directory");
            return None;
        }

        if !Path::new(&format!("{base_dir}/startnb.sh")).is_file() {
            println!("Couldn't find the startnb.sh script in '{base_dir}'");
            return None;
        } else if !Path::new(&format!("{base_dir}/etc/")).is_dir() {
            println!("Couldn't find a 'etc/' directory in '{base_dir}'");
            return None;
        }
        Some(base_dir)
    } else {
        unreachable!(); // Because we already checked that args_iter.len() == 2
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_dir = get_base_dir_arg().ok_or("Failed to find mandatory files or directories")?;

    let terminal = ratatui::init();
    let (tx, rx) = std::sync::mpsc::channel::<AppEvent>();

    let app_state = app::State::new(base_dir, tx.clone())?;

    // Starting a thread to listen to key events
    std::thread::spawn(move || {
        loop {
            let event = crossterm::event::read().unwrap();
            match event {
                crossterm::event::Event::FocusGained => (),
                crossterm::event::Event::FocusLost => (),
                crossterm::event::Event::Key(key_event) => {
                    tx.send(AppEvent::Key(key_event)).unwrap()
                }
                crossterm::event::Event::Mouse(_) => (),
                crossterm::event::Event::Paste(_) => (),
                crossterm::event::Event::Resize(_, _) => (),
            }
        }
    });

    let result = run(terminal, app_state, rx);
    ratatui::restore();
    result
}

fn run(
    mut terminal: DefaultTerminal,
    mut app_state: app::State,
    rx: Receiver<AppEvent>,
) -> Result<(), Box<dyn std::error::Error>> {
    while !app_state.exit {
        terminal.draw(|frame| ui::render(frame, &mut app_state))?;

        // Wait for an event
        let event = rx.recv().unwrap();

        if events::handle(&mut app_state, event).is_err() {
            // TODO: better error handling ?
            break;
        }
    }

    Ok(())
}
