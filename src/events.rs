use ratatui::{
    crossterm::event::{self, KeyCode, KeyEvent},
    widgets::ScrollbarState,
};

use crate::app::{Screen, StartStopState, State};

pub enum AppEvent {
    Key(KeyEvent),
    StartNbOutput(Result<std::process::Output, std::io::Error>),
}

pub fn handle(app_state: &mut State, event: AppEvent) -> Result<(), Box<dyn std::error::Error>> {
    match event {
        AppEvent::Key(key_event) if key_event.kind == event::KeyEventKind::Press => {
            match app_state.current_screen {
                Screen::List => match key_event.code {
                    KeyCode::Down => {
                        app_state.table_state.select_next();
                    }
                    KeyCode::Up => {
                        app_state.table_state.select_previous();
                    }
                    KeyCode::Home => {
                        app_state.table_state.select_first();
                    }
                    KeyCode::End => {
                        app_state.table_state.select_last();
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
                },
                Screen::StartStop(ref mut _start_stop_state) => {
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
                Screen::DeleteConfirmation(ok) => match key_event.code {
                    KeyCode::Esc => {
                        app_state.current_screen = Screen::List;
                    }
                    KeyCode::Left => app_state.current_screen = Screen::DeleteConfirmation(true),
                    KeyCode::Right => app_state.current_screen = Screen::DeleteConfirmation(false),
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
                },
            }
        }
        AppEvent::Key(_) => {}
        AppEvent::StartNbOutput(output) => {
            match output {
                Ok(res) => {
                    if res.status.success() {
                        // Updating the VM info
                        //current_vm.update_pid(&app_state.base_dir);
                        app_state.refresh();
                    } else {
                        let err_str = format!(
                            "startnb.sh failed!\n{}{}",
                            String::from_utf8(res.stdout).unwrap(),
                            String::from_utf8(res.stderr).unwrap()
                        );
                        app_state.current_screen = Screen::StartStop(StartStopState {
                            err_str: Some(err_str),
                            vertical_scroll_bar_pos: 0,
                            vertical_scroll_bar_state: ScrollbarState::default(),
                        });
                    }
                }
                Err(err) => {
                    let err_str = format!("std::process::Command::new() failed!\n{err}");
                    app_state.current_screen = Screen::StartStop(StartStopState {
                        err_str: Some(err_str),
                        vertical_scroll_bar_pos: 0,
                        vertical_scroll_bar_state: ScrollbarState::default(),
                    });
                }
            }
        }
    }

    Ok(())
}
