use ratatui::crossterm::event::KeyEvent;

#[derive(Debug)]
pub enum AppEvent {
    ForceRender,
    Key(KeyEvent),
    StartNbFailed {
        vm_name: String,
        error: String,
        stdout: String,
        stderr: String,
    },
    StartNbSuccess {
        vm_name: String,
    },
    KillFailed {
        vm_name: String,
        error: String,
    },
    FatalError(String),
    VmConfCreated(String),
    VmConfModified(String),
    VmConfDeleted(String),
    KernelCreated(String),
    KernelModified(String),
    KernelDeleted(String),
    PidFileDeleted(String),
    PidFileCreated(String),
    ImageFileCreated(String),
    ImageFileModified(String),
    ImageFileDeleted(String),
}
