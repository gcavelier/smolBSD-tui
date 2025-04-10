use libc::{SIGTERM, c_int, kill, strerror};
use ratatui::widgets::{ScrollbarState, TableState};
use std::{
    collections::HashMap,
    ffi::CStr,
    fs::{DirEntry, read_to_string},
    path::Path,
};

#[derive(Clone, PartialEq)]
pub struct StartStopState {
    pub err_str: Option<String>,
    pub vertical_scroll_bar_pos: usize,
    // TODO: implement a max position (err_str.lines().count() - height of the top_chunk - margins)
    pub vertical_scroll_bar_state: ScrollbarState,
}
#[derive(Clone, PartialEq)]
pub enum Screen {
    /// VMs List
    List,
    /// This screen is only used to display an error when starting or stopping a VM
    StartStop(StartStopState),
    /// Confirmation popup when deleting a VM. The boolean value indicates if "OK" has been selected
    DeleteConfirmation(bool),
}

pub struct Vm {
    /// Name of the config file without the '.conf' extension
    pub name: String,
    config_data: HashMap<String, String>,
    pub pid: Option<i32>,
    pub cpu_usage: u8,
}

impl Vm {
    fn update_pid(&mut self, base_directory: &str) {
        let pid_file = format!("{}/qemu-{}.pid", base_directory, self.name);
        self.pid = match Path::new(&pid_file).exists() {
            false => None,
            true => match read_to_string(pid_file) {
                Ok(res) => res.trim().parse().ok(),
                Err(_) => None,
            },
        }
    }
    fn kill(&mut self) -> Result<(), String> {
        match self.pid {
            Some(pid) => {
                let res: c_int;
                let err_str: &str;
                unsafe {
                    res = kill(pid, SIGTERM);
                    err_str = CStr::from_ptr(strerror(res))
                        .to_str()
                        .unwrap_or("utf8 error when convertir error string for libc");
                }
                if res == 0 {
                    self.pid = None;
                    Ok(())
                } else {
                    Err(format!("Failed to kill PID {pid}: {err_str}"))
                }
            }
            None => Ok(()),
        }
    }
}

pub struct State {
    base_dir: String,
    pub ms_elapsed: u64,
    pub vms: Vec<Vm>,
    pub kernels: Option<Vec<DirEntry>>,
    pub images: Option<Vec<DirEntry>>,
    pub table_state: TableState,
    pub current_screen: Screen,
    pub exit: bool,
}

impl State {
    pub fn new(base_dir: String) -> Result<Self, Box<dyn std::error::Error>> {
        let mut state = State {
            base_dir,
            ms_elapsed: 0,
            vms: Vec::new(),
            kernels: None,
            images: None,
            table_state: TableState::default(),
            current_screen: Screen::List,
            exit: false,
        };
        state.refresh();
        if !state.vms.is_empty() {
            state.table_state.select(Some(0));
        }
        Ok(state)
    }
    pub fn refresh(&mut self) {
        let kernels = files_in_directory(&format!("{}/kernels", &self.base_dir)).ok();
        let images = files_in_directory(&format!("{}/images", &self.base_dir)).ok();
        let mut vms = get_vms(&self.base_dir).unwrap_or_else(|_| vec![]);
        vms.sort_by(|vm1, vm2| vm1.name.cmp(&vm2.name));
        self.kernels = kernels;
        self.images = images;
        self.vms = vms;
    }
    pub fn start_stop_vm(&mut self) -> Result<(), String> {
        let current_vm_idx = self.table_state.selected().ok_or("No VM selected!")?;
        let current_vm = self
            .vms
            .get_mut(current_vm_idx)
            .ok_or(format!("Could not get_mut() VM at index {current_vm_idx}"))?;

        match current_vm.pid {
            Some(_) => {
                return current_vm.kill();
            }
            None => {
                match std::process::Command::new(
                    std::fs::canonicalize(format!("{}/startnb.sh", self.base_dir))
                        .map_err(|err| format!("std::fs::canonicalize() failed: {err}"))?,
                )
                .args(["-f", &format!("etc/{}.conf", current_vm.name), "-d"])
                .current_dir(&self.base_dir)
                .output()
                {
                    Ok(res) => {
                        if res.stdout.is_empty() && res.stderr.is_empty() {
                            // Updating the VM info
                            current_vm.update_pid(&self.base_dir);
                        } else {
                            let err_str = format!(
                                "startnb.sh failed!\n{}{}",
                                String::from_utf8(res.stdout).unwrap(),
                                String::from_utf8(res.stderr).unwrap()
                            );
                            return Err(err_str);
                        }
                    }
                    Err(err) => {
                        let err_str = format!("std::process::Command::new() failed!\n{err}");
                        return Err(err_str);
                    }
                }
            }
        }

        Ok(())
    }

    pub fn delete_vm(&mut self) {
        let current_vm_idx = match self.table_state.selected() {
            Some(idx) => idx,
            None => return, // This should never happened
        };
        let current_vm = match self.vms.get_mut(current_vm_idx) {
            Some(vm) => vm,
            None => return, // This should never happened
        };
        current_vm.pid.map(|_| current_vm.kill());
        let file_to_delete = format!("{}/etc/{}.conf", self.base_dir, current_vm.name);
        std::fs::remove_file(&file_to_delete)
            .unwrap_or_else(|_| panic!("Couldn't delete file {}", file_to_delete));
        self.vms.remove(current_vm_idx);
        self.table_state
            .select(Some(current_vm_idx.min(self.vms.len() - 1)));
    }
}

fn get_vms(base_directory: &str) -> Result<Vec<Vm>, Box<dyn std::error::Error>> {
    let conf_directory = format!("{base_directory}/etc");

    let vm_confs = files_in_directory(&conf_directory)
        .ok()
        .map_or(vec![], |vm_confs| {
            vm_confs
                .iter()
                .filter(|vm_conf_file| {
                    // filename must ends with ".conf"
                    vm_conf_file
                        .file_name()
                        .to_string_lossy()
                        .ends_with(".conf")
                })
                .filter_map(|vm_conf_file| {
                    // Converts file into a HashMap
                    let vm_conf = std::fs::read_to_string(vm_conf_file.path()).ok()?;
                    let hashmap: HashMap<String, String> = vm_conf
                        .lines()
                        .filter(|line| {
                            // Filter out those lines
                            !line.starts_with('#')
                                && !line.starts_with("extra")
                                && line.contains('=')
                        })
                        .map(|line| {
                            // We already checked that 'line' contains '=', so calling unwrap() is ok
                            let (key, value) = line.trim().split_once('=').unwrap();
                            (key.to_owned(), value.to_owned())
                        })
                        .collect();
                    // If the hashmap doesn't contain the 'vm' key, we discard it
                    hashmap.contains_key("vm").then(|| {
                        (
                            hashmap,
                            vm_conf_file
                                .file_name()
                                .into_string()
                                .unwrap_or("".to_string()),
                        )
                    })
                })
                .map(|(config_data, vm_conf_file)| {
                    let vm_conf_file = vm_conf_file.strip_suffix(".conf").unwrap(); // We filtered the files ending with '.conf', so this unwrap() always succeed
                    let mut vm = Vm {
                        pid: None,
                        name: vm_conf_file.to_string(),
                        config_data,
                        cpu_usage: 0,
                    };
                    vm.update_pid(base_directory);
                    vm
                })
                .collect()
        });
    Ok(vm_confs)
}

fn files_in_directory(directory: &str) -> Result<Vec<DirEntry>, Box<dyn std::error::Error>> {
    let res: Vec<_> = std::fs::read_dir(directory)?
        .filter_map(|res_dir_entry| res_dir_entry.ok())
        .filter(|dir_entry| dir_entry.file_type().is_ok_and(|entry| entry.is_file()))
        .collect();
    Ok(res)
}
