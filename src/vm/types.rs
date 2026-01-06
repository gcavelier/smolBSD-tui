use std::collections::HashMap;
use std::ffi::CStr;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

use libc::c_int;
use ratatui::style::Color;

use crate::ui::{INVALID_CONF_VM_FG, RUNNING_VM_FG, STARTING_VM_FG, STOPPED_VM_FG, STOPPING_VM_FG};
use crate::vm;

#[derive(Debug)]
pub enum VmState {
    InvalidConfiguration { cause: String },
    Starting,
    Running { pid: u32 },
    Stopping,
    StoppingToDelete,
    Stopped,
}

#[derive(Debug)]
pub struct Vm {
    ///
    /// Mandatory parameters
    ///

    /// Name of the config file without the '.conf' extension
    pub name: String,
    /// Disk image
    pub img: Option<String>,
    /// Kernel to load
    pub kernel: Option<String>,

    ///
    /// Optional parameters
    ///
    pub mem: Option<String>,
    pub cores: Option<u8>,
    pub hostfwd: Option<String>,
    pub editprotect: bool,
    pub rmprotect: bool,
    pub qmp_port: Option<u16>,
    pub bridgenet: Option<String>,
    pub share: Option<String>,
    pub sharerw: bool,
    pub extra: Option<String>,

    /// State
    pub state: VmState,
    pub cpu_usage: u8,
}

impl Vm {
    pub fn new(vm_conf: Vec<(&str, &str)>, conf_file: &PathBuf) -> Self {
        // This will be the returned struct
        let mut res = Vm {
            name: conf_file
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .strip_suffix(".conf")
                // SAFETY: this unwrap always succeed (get_vms() filtered the .conf files)
                .unwrap()
                .to_owned(),
            img: None,
            kernel: None,
            mem: None,
            cores: None,
            hostfwd: None,
            editprotect: false,
            rmprotect: false,
            qmp_port: None,
            bridgenet: None,
            share: None,
            sharerw: false,
            extra: None,
            state: VmState::Stopped,
            cpu_usage: 0,
        };

        // Convert vm_conf into a hashmap to check if it contains all the mandatory keys
        let mut vm_conf_hashmap: HashMap<&str, &str> = HashMap::new();
        for (key, value) in &vm_conf {
            vm_conf_hashmap.insert(*key, *value);
        }

        // Check if all the mandatory keys are present
        // None for now! ;)

        for (key, value) in vm_conf {
            match key {
                "img" => res.img = Some(value.to_owned()),
                "kernel" => res.kernel = Some(value.to_owned()),
                "mem" => res.mem = Some(value.to_owned()),
                "cores" => {
                    res.cores = match value.parse() {
                        Ok(value) => Some(value),
                        Err(err) => {
                            res.state = VmState::InvalidConfiguration {
                                cause: format!(
                                    "Failed to convert 'cores' parameter ({value}) to a u8: {err}"
                                ),
                            };
                            break;
                        }
                    }
                }
                "hostfwd" => res.hostfwd = Some(value.to_owned()),
                "editprotect" => {
                    res.editprotect = match vm::helpers::parse_bool(value) {
                        Ok(value) => value,
                        Err(err) => {
                            res.state = VmState::InvalidConfiguration {
                                cause: format!(
                                    "Failed to parse 'editprotect' parameter ({value}) to a boolean: {err}"
                                ),
                            };
                            break;
                        }
                    }
                }
                "rmprotect" => {
                    res.rmprotect = match vm::helpers::parse_bool(value) {
                        Ok(value) => value,
                        Err(err) => {
                            res.state = VmState::InvalidConfiguration {
                                cause: format!(
                                    "Failed to parse 'rmprotect' parameter ({value}) to a boolean: {err}"
                                ),
                            };
                            break;
                        }
                    }
                }
                "qmp_port" => res.qmp_port = Some(value.parse().unwrap_or(0)),
                "bridgenet" => res.bridgenet = Some(value.to_owned()),
                "share" => res.share = Some(value.to_owned()),
                "sharerw" => {
                    res.sharerw = match vm::helpers::parse_bool(value) {
                        Ok(value) => value,
                        Err(err) => {
                            res.state = VmState::InvalidConfiguration {
                                cause: format!(
                                    "Failed to parse 'sharerw' parameter ({value}) to a boolean: {err}"
                                ),
                            };
                            break;
                        }
                    }
                }
                "extra" => res.extra = Some(value.to_owned()),
                _ => {}
            }
        }

        res
    }

    pub fn state(&self) -> (String, Color) {
        match self.state {
            VmState::InvalidConfiguration { .. } => {
                ("Invalid configuration".to_owned(), INVALID_CONF_VM_FG)
            }
            VmState::Starting => ("Starting...".to_owned(), STARTING_VM_FG),
            VmState::Running { .. } => ("Running".to_owned(), RUNNING_VM_FG),
            VmState::Stopping => ("Stopping".to_owned(), STOPPING_VM_FG),
            VmState::StoppingToDelete => ("Stopping".to_owned(), STOPPING_VM_FG),
            VmState::Stopped => ("Stopped".to_owned(), STOPPED_VM_FG),
        }
    }

    pub fn update_state(&mut self, base_directory: &str) {
        match &self.state {
            VmState::Starting | VmState::Running { .. } | VmState::Stopped | VmState::Stopping => {
                self.set_pid(base_directory);
            }
            // We don't do anything in those cases
            VmState::InvalidConfiguration { .. } | VmState::StoppingToDelete => {}
        }
    }

    pub fn kill(&mut self) -> Result<(), String> {
        match self.state {
            VmState::Running { pid } => {
                let res: c_int;
                let err_str: &str;
                unsafe {
                    res = libc::kill(pid as i32, libc::SIGTERM);
                    err_str = CStr::from_ptr(libc::strerror(res))
                        .to_str()
                        .unwrap_or("utf8 error when convertir error string for libc");
                }
                if res == 0 {
                    self.state = VmState::Stopping;
                    Ok(())
                } else {
                    Err(format!("Failed to kill PID {pid}: {err_str}"))
                }
            }
            _ => Ok(()),
        }
    }

    pub fn is_running(&self) -> bool {
        matches!(self.state, VmState::Running { .. })
    }

    pub fn set_pid(&mut self, base_directory: &str) {
        match &self.state {
            VmState::Stopped | VmState::Starting => {
                let pid_file = format!("{}/qemu-{}.pid", base_directory, self.name);
                if let Ok(res) = std::fs::exists(&pid_file)
                    && res == true
                {
                    self.state = match read_to_string(&pid_file) {
                        Ok(res) => match res.trim().parse() {
                            Ok(res) => VmState::Running { pid: res },
                            Err(err) => VmState::InvalidConfiguration {
                                cause: format!("Failed to parse pid file {pid_file}: {err}"),
                            },
                        },
                        Err(err) => VmState::InvalidConfiguration {
                            cause: format!("Failed to read pid file {pid_file}: {err}"),
                        },
                    }
                } else {
                    self.state = VmState::Stopped
                }
            }
            _ => {}
        }
    }
}
