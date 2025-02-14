use std::time::Duration;

use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    widgets::ScrollbarState,
};

const POLL_INTERVAL_MS: u64 = 100;
const LIST_REFRESH_INTERVAL_SEC: u64 = 2;

use crate::app::{Screen, StartStopState, State};

pub fn handle(app_state: &mut State) -> Result<(), Box<dyn std::error::Error>> {
    // We wait POLL_INTERVAL_MS for a key
    let key = if event::poll(Duration::from_millis(POLL_INTERVAL_MS))? {
        event::read()?
    } else {
        let (ms_elapsed, _) = app_state.ms_elapsed.overflowing_add(POLL_INTERVAL_MS);
        app_state.ms_elapsed = ms_elapsed;
        if app_state.ms_elapsed % (LIST_REFRESH_INTERVAL_SEC * 1000) == 0 {
            app_state.refresh();
        }
        return Ok(());
    };

    match app_state.current_screen {
        Screen::List => match key {
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
                            app_state.current_screen = Screen::StartStop(StartStopState {
                                err_str: None,
                                vertical_scroll_bar_pos: 0,
                                vertical_scroll_bar_state: ScrollbarState::default(),
                            })
                        }
                        KeyCode::Char('d') => {
                            app_state.current_screen = Screen::DeleteConfirmation(false);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        },
        Screen::StartStop(ref mut _start_stop_state) => match key {
            Event::Key(key_event) => {
                if key_event.kind == event::KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Esc | KeyCode::Enter => app_state.current_screen = Screen::List,
                        // TODO: uncomment when max scroll position is handled
                        // KeyCode::Down => {
                        //     let pos = start_stop_state.vertical_scroll_bar_pos.saturating_add(1);
                        //     start_stop_state.vertical_scroll_bar_pos = pos;
                        //     start_stop_state.vertical_scroll_bar_state =
                        //         start_stop_state.vertical_scroll_bar_state.position(pos);
                        // }
                        // KeyCode::Up => {
                        //     let pos = start_stop_state.vertical_scroll_bar_pos.saturating_sub(1);
                        //     start_stop_state.vertical_scroll_bar_pos = pos;
                        //     start_stop_state.vertical_scroll_bar_state =
                        //         start_stop_state.vertical_scroll_bar_state.position(pos);
                        // }
                        _ => {}
                    }
                }
            }
            _ => {}
        },
        Screen::DeleteConfirmation(ok) => match key {
            Event::Key(key_event) => {
                if key_event.kind == event::KeyEventKind::Press {
                    match key_event.code {
                        KeyCode::Esc => {
                            app_state.current_screen = Screen::List;
                        }
                        KeyCode::Left => {
                            app_state.current_screen = Screen::DeleteConfirmation(true)
                        }
                        KeyCode::Right => {
                            app_state.current_screen = Screen::DeleteConfirmation(false)
                        }
                        KeyCode::Tab => {
                            app_state.current_screen = Screen::DeleteConfirmation(!ok);
                        }
                        KeyCode::Enter => {
                            if ok {
                                app_state.delete_vm()
                            }
                            app_state.current_screen = Screen::List;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        },
    }

    Ok(())
}
