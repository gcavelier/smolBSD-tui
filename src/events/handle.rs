use crate::{app::State, events::AppEvent, ui::Screen, vm::VmState};
use ratatui::crossterm::event::{self, KeyCode};

pub fn handle(app: &mut State, event: AppEvent) -> Result<(), Box<dyn std::error::Error>> {
    match event {
        AppEvent::Key(key_event) if key_event.kind == event::KeyEventKind::Press => {
            match app.current_screen {
                Screen::List => match key_event.code {
                    KeyCode::Down => {
                        app.table_state.select_next();
                    }
                    KeyCode::Up => {
                        app.table_state.select_previous();
                    }
                    KeyCode::Home => {
                        app.table_state.select_first();
                    }
                    KeyCode::End => {
                        app.table_state.select_last();
                    }
                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                        app.exit = true;
                    }
                    KeyCode::Char('s') => {
                        app.start_stop_selected_vm();
                    }
                    KeyCode::Char('d') => {
                        app.current_screen = Screen::DeleteConfirmation(false);
                    }
                    _ => {}
                },
                Screen::DeleteConfirmation(ok) => match key_event.code {
                    KeyCode::Esc => {
                        app.current_screen = Screen::List;
                    }
                    KeyCode::Left => app.current_screen = Screen::DeleteConfirmation(true),
                    KeyCode::Right => app.current_screen = Screen::DeleteConfirmation(false),
                    KeyCode::Tab => {
                        app.current_screen = Screen::DeleteConfirmation(!ok);
                    }
                    KeyCode::Enter => {
                        if ok {
                            app.delete_selected_vm();
                        }
                        app.current_screen = Screen::List;
                    }
                    _ => {}
                },
                Screen::StartNbFailed { .. } => match key_event.code {
                    KeyCode::Esc | KeyCode::Enter => {
                        app.current_screen = Screen::List;
                    }
                    _ => {}
                },
                Screen::KillFailed { .. } => match key_event.code {
                    KeyCode::Esc | KeyCode::Enter => {
                        app.current_screen = Screen::List;
                    }
                    _ => {}
                },
            }
        }

        AppEvent::Key(_) => {}

        AppEvent::StartNbSuccess { vm_name } => {
            let base_dir = app.base_dir.clone();
            if let Some(vm) = app.get_mut_vm_by_name(&vm_name) {
                vm.update_state(&base_dir);
            };
        }

        AppEvent::StartNbFailed {
            vm_name,
            error,
            stdout,
            stderr,
        } => {
            if let Some(vm) = app.get_mut_vm_by_name(&vm_name) {
                vm.state = VmState::Stopped;
            }
            app.current_screen = Screen::StartNbFailed {
                vm_name,
                error,
                stdout,
                stderr,
            }
        }

        AppEvent::ForceRender => {}

        AppEvent::KillFailed { vm_name, error } => {
            app.current_screen = Screen::KillFailed { vm_name, error }
        }
        AppEvent::FatalError(err) => app.fatal_error = Some(err),

        AppEvent::VmConfCreated(filename) => {
            app.add_vm(&filename);
        }
        AppEvent::VmConfModified(filename) => {}
        AppEvent::VmConfDeleted(filename) => {
            app.delete_vm(&filename);
        }
        AppEvent::KernelCreated(filename) => {}
        AppEvent::KernelModified(filename) => {}
        AppEvent::KernelDeleted(filename) => {}
        AppEvent::PidFileDeleted(vm_name) => {
            if let Some(vm) = app.get_mut_vm_by_name(&vm_name) {
                match vm.state {
                    VmState::StoppingToDelete => app.vms.retain(|item| item.name != vm_name),
                    _ => vm.state = VmState::Stopped,
                }
            }
        }
        AppEvent::ImageFileCreated(filename) => {}
        AppEvent::ImageFileModified(filename) => {}
        AppEvent::ImageFileDeleted(filename) => {}
    }

    Ok(())
}
