use crate::app::args;
use crate::events::AppEvent;
use crate::ui::{LOGO, Screen};
use crate::vm::{self, Vm, VmState};
use ratatui::widgets::TableState;
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
use std::fs::DirEntry;
use std::path::PathBuf;
use std::sync::mpsc::Sender;

pub struct State {
    pub base_dir: String,
    pub vms: Vec<Vm>,
    pub kernels: Option<Vec<DirEntry>>,
    pub images: Option<Vec<DirEntry>>,
    pub table_state: TableState,
    pub current_screen: Screen,
    pub exit: bool,
    pub fatal_error: Option<String>,
    pub tx: Sender<AppEvent>,
    pub logo: StatefulProtocol,
}

impl State {
    pub fn new(tx: Sender<AppEvent>) -> Result<Self, Box<dyn std::error::Error>> {
        let base_dir =
            args::get_base_dir().ok_or("Failed to find mandatory files or directories")?;

        let mut vms = vm::helpers::get_vms(&base_dir)
            .map_err(|err| eprintln!("Failed to read VMs configurations: {err}"))
            .unwrap();

        // Sort VMs by name
        vms.sort_by(|vm1, vm2| vm1.name.cmp(&vm2.name));

        let picker = Picker::from_query_stdio()?;
        // TODO: picker.protocol_type() to know if the terminal supports images
        let dyn_logo = image::load_from_memory(LOGO)?;
        let logo = picker.new_resize_protocol(dyn_logo);

        Ok(Self {
            kernels: vm::helpers::files_in_directory(&format!("{}/kernels", &base_dir)).ok(),
            images: vm::helpers::files_in_directory(&format!("{}/images", &base_dir)).ok(),
            table_state: if vms.is_empty() {
                TableState::default()
            } else {
                TableState::with_selected(TableState::default(), 0)
            },
            current_screen: Screen::List,
            exit: false,
            fatal_error: None,
            tx,
            base_dir,
            vms,
            logo,
        })
    }

    #[must_use]
    pub fn get_mut_vm_by_name(&mut self, name: &str) -> Option<&mut Vm> {
        self.vms.iter_mut().find(|item| item.name.as_str() == name)
    }

    /// This function starts or stops the currently selected VM depending on its state
    ///
    /// ⚠️ this function is called from the events handling loop, so it **must** be quick! (that's why it starts a thread when necessary)
    pub fn start_stop_selected_vm(&mut self) {
        if let Some(select_vm_idx) = self.table_state.selected()
            && let Some(selected_vm) = self.vms.get_mut(select_vm_idx)
        {
            match &mut selected_vm.state {
                VmState::InvalidConfiguration { .. }
                | VmState::Starting
                | VmState::Stopping
                | VmState::StoppingToDelete => {
                    // We don't do anything in thoses cases
                }
                VmState::Running { .. } => {
                    let _ = selected_vm.kill().map_err(|err| {
                        self.tx
                            .send(AppEvent::KillFailed {
                                vm_name: selected_vm.name.clone(),
                                error: err,
                            })
                            .unwrap()
                    });
                }
                VmState::Stopped => {
                    selected_vm.state = VmState::Starting;

                    // We have to clone those variables because they will be used by the thread created below
                    let tx = self.tx.clone();
                    let base_dir = self.base_dir.clone();
                    let selected_vm_name = selected_vm.name.clone();

                    // Starting a new thread!
                    std::thread::spawn(move || {
                        // TODO: move the following code in selected_vm.start()
                        let startnb_path =
                            match std::fs::canonicalize(format!("{}/startnb.sh", base_dir)) {
                                Ok(value) => value,
                                Err(err) => {
                                    tx.send(AppEvent::StartNbFailed {
                                        vm_name: selected_vm_name,
                                        error: format!("std::fs::canonicalize() failed: {err}"),
                                        stdout: String::new(),
                                        stderr: String::new(),
                                    })
                                    .unwrap();
                                    return;
                                }
                            };
                        let startnb_output = std::process::Command::new(startnb_path)
                            .args(["-f", &format!("etc/{}.conf", selected_vm_name), "-d"])
                            .current_dir(&base_dir)
                            .output();

                        // Sending the result through tx
                        match startnb_output {
                            Ok(output) => {
                                if output.status.success() {
                                    tx.send(AppEvent::StartNbSuccess {
                                        vm_name: selected_vm_name,
                                    })
                                    .unwrap()
                                } else {
                                    tx.send(AppEvent::StartNbFailed {
                                        vm_name: selected_vm_name,
                                        error: "startnb.sh failed!".to_owned(),
                                        stdout: String::from_utf8(output.stdout).unwrap(),
                                        stderr: String::from_utf8(output.stderr).unwrap(),
                                    })
                                    .unwrap()
                                }
                            }
                            Err(err) => tx
                                .send(AppEvent::StartNbFailed {
                                    vm_name: selected_vm_name,
                                    error: format!("startnb.sh failed: {err}"),
                                    stdout: String::new(),
                                    stderr: String::new(),
                                })
                                .unwrap(),
                        }
                    });
                }
            }
        }
    }

    pub fn delete_selected_vm(&mut self) {
        if let Some(selected_vm_idx) = self.table_state.selected()
            && let Some(selected_vm) = self.vms.get_mut(selected_vm_idx)
        {
            if selected_vm.is_running() {
                // The VM is running, we must kill it first!
                let _ = selected_vm.kill().map_err(|err| {
                    self.tx
                        .send(AppEvent::KillFailed {
                            vm_name: selected_vm.name.clone(),
                            error: err,
                        })
                        .unwrap()
                });
                selected_vm.state = VmState::StoppingToDelete;
                // The VM will be deleted when the PID file will be deleted, not right now
            } else {
                // The VM is not running, we can delete it right away
                let file_to_delete = format!("{}/etc/{}.conf", self.base_dir, selected_vm.name);
                std::fs::remove_file(&file_to_delete)
                    .unwrap_or_else(|_| panic!("Couldn't delete file {}", file_to_delete));
                self.vms.remove(selected_vm_idx);
                self.table_state.select(match self.vms.is_empty() {
                    true => None,
                    false => Some(selected_vm_idx.min(self.vms.len() - 1)),
                });
            }
        }
    }

    /// `conf_file` **must** be an absolute path
    pub fn add_vm(&mut self, conf_file: &str) {
        let relative_conf_file = conf_file.strip_prefix(&self.base_dir).unwrap();

        if let Some(vm_name) = relative_conf_file
            .strip_prefix("etc/")
            .and_then(|value| value.strip_suffix(".conf"))
        {
            if self.get_mut_vm_by_name(vm_name).is_none() {
                // This VM doesn't already exist, we can create it
                let conf_file = PathBuf::from(conf_file);
                if let Ok(vm) = vm::helpers::vm_from_conf(conf_file, &self.base_dir) {
                    self.vms.push(vm);
                    // Sort VMs by name
                    self.vms.sort_by(|vm1, vm2| vm1.name.cmp(&vm2.name));
                }
            }
        }
    }

    /// `conf_file` **must** be an absolute path
    pub fn delete_vm(&mut self, conf_file: &str) {
        let relative_conf_file = conf_file.strip_prefix(&self.base_dir).unwrap();

        if let Some(vm_name) = relative_conf_file
            .strip_prefix("etc/")
            .and_then(|value| value.strip_suffix(".conf"))
        {
            if let Some(vm) = self
                .vms
                .iter_mut()
                .find(|item| item.name.as_str() == vm_name)
            {
                if vm.is_running() {
                    // The VM is running, we must kill it first!
                    let _ = vm.kill().map_err(|err| {
                        self.tx
                            .send(AppEvent::KillFailed {
                                vm_name: vm.name.clone(),
                                error: err,
                            })
                            .unwrap()
                    });
                    vm.state = VmState::StoppingToDelete;
                    // The VM will be deleted when the PID file will be deleted, not right now
                } else {
                    // The VM is not running, we can delete it right away
                    self.vms.retain(|item| item.name != vm_name);
                }
            }
        }
    }
}
