use notify::{Event, RecursiveMode, Result, Watcher};
use std::{
    path::{Path, PathBuf},
    sync::mpsc::{self, Sender},
};

use crate::events::AppEvent;

#[derive(Debug)]
enum FileOperation {
    Created,
    Modified,
    Deleted,
}

pub fn get_fs_events(app_tx: Sender<AppEvent>, base_dir: String) {
    let (tx, rx) = mpsc::channel::<Result<Event>>();

    // Use recommended_watcher() to automatically select the best implementation
    let mut watcher = notify::recommended_watcher(tx).unwrap();

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.
    watcher
        .watch(Path::new(&base_dir), RecursiveMode::Recursive)
        .unwrap();

    // Block forever, printing out events as they come in
    for res in rx {
        match res {
            Ok(event) => match event.kind {
                notify::EventKind::Any
                | notify::EventKind::Access(_)
                | notify::EventKind::Other => {}
                notify::EventKind::Create(create_kind) => match create_kind {
                    notify::event::CreateKind::File => {
                        send_file_event(&base_dir, &app_tx, event.paths, FileOperation::Created)
                    }
                    _ => {}
                },
                notify::EventKind::Modify(modify_kind) => match modify_kind {
                    notify::event::ModifyKind::Data(_) => {
                        send_file_event(&base_dir, &app_tx, event.paths, FileOperation::Modified)
                    }
                    _ => {}
                },
                notify::EventKind::Remove(remove_kind) => match remove_kind {
                    notify::event::RemoveKind::File => {
                        send_file_event(&base_dir, &app_tx, event.paths, FileOperation::Deleted)
                    }
                    _ => {}
                },
            },
            Err(e) => eprintln!("watch error: {:?}", e),
        }
    }
}

/// Depending on the filenames in `paths`, sends the corresponding AppEvent
fn send_file_event(
    base_dir: &str,
    app_tx: &Sender<AppEvent>,
    paths: Vec<PathBuf>,
    operation: FileOperation,
) {
    eprintln!("operation: {operation:?}, paths: {paths:?}");

    if paths.len() > 1 {
        // FIXME: does this ever happen ?
        app_tx
            .send(AppEvent::FatalError(format!(
                "send_file_event(), {operation:?}: paths.len() > 1, paths={paths:?}",
            )))
            .unwrap();
        return;
    }

    if let Some(path) = paths.into_iter().nth(0) {
        if let Ok(filename) = path
            .strip_prefix(base_dir)
            .map(|value| value.to_str().unwrap_or(""))
        {
            let event = if let Some(filename) = filename.strip_prefix("etc/") {
                // VM configuration file
                let filename = filename.to_owned();
                Some(match operation {
                    FileOperation::Created => AppEvent::VmConfCreated(filename),
                    FileOperation::Modified => AppEvent::VmConfModified(filename),
                    FileOperation::Deleted => AppEvent::VmConfDeleted(filename),
                })
            } else if let Some(filename) = filename.strip_prefix("images/") {
                // image file
                let filename = filename.to_owned();
                Some(match operation {
                    FileOperation::Created => AppEvent::ImageFileCreated(filename),
                    FileOperation::Modified => AppEvent::ImageFileModified(filename),
                    FileOperation::Deleted => AppEvent::ImageFileDeleted(filename),
                })
            } else if let Some(filename) = filename.strip_prefix("kernels/") {
                // kernel file
                let filename = filename.to_owned();
                Some(match operation {
                    FileOperation::Created => AppEvent::KernelCreated(filename),
                    FileOperation::Modified => AppEvent::KernelModified(filename),
                    FileOperation::Deleted => AppEvent::KernelDeleted(filename),
                })
            } else if let Some(vmname) = filename.strip_prefix("qemu-")
                && let Some(vmname) = vmname.strip_suffix(".pid")
            {
                // QEMU PID file
                let vmname = vmname.to_owned();
                match operation {
                    FileOperation::Created => None,
                    FileOperation::Modified => None,
                    FileOperation::Deleted => Some(AppEvent::PidFileDeleted(vmname)),
                }
            } else {
                None
            };

            if let Some(event) = event {
                app_tx.send(event).unwrap();
            }
        }
    }
}
