use ratatui::crossterm::event::{self, Event, KeyCode};

use crate::app::State;

pub fn handle(app_state: &mut State) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: use polling to update the (not yet implemented) timer
    // https://ratatui.rs/tutorials/counter-async-app/async-event-stream/
    let key = event::read()?;

    match app_state.current_screen {
        crate::app::CurrentScreen::List => match key {
            Event::Key(key_event) => {
                if key_event.kind == event::KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Down => {
                            app_state.selected_vm_idx =
                                (app_state.selected_vm_idx + 1).min(app_state.vms.len() - 1);
                        }
                        KeyCode::Up => {
                            app_state.selected_vm_idx = match app_state.selected_vm_idx {
                                0 => 0,
                                _ => app_state.selected_vm_idx - 1,
                            };
                        }
                        KeyCode::Home => {
                            app_state.selected_vm_idx = 0;
                        }
                        KeyCode::End => {
                            app_state.selected_vm_idx = app_state.vms.len() - 1;
                        }
                        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                            app_state.exit = true;
                        }
                        KeyCode::Char('s') => {
                            app_state.current_screen = crate::app::CurrentScreen::StartStop
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        },
        crate::app::CurrentScreen::StartStop => match key {
            Event::Key(key_event) => {
                if key_event.kind == event::KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Esc => app_state.current_screen = crate::app::CurrentScreen::List,
                        _ => {}
                    }
                }
            }
            _ => {}
        },
    }

    Ok(())
}
