use libc::{c_int, kill, strerror, SIGTERM};
use ratatui::widgets::{ScrollbarState, TableState};
use std::{
    collections::HashMap,
    ffi::CStr,
    fs::{read_to_string, DirEntry},
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

#[derive(Debug)]
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
                Ok(res) => match res.trim().parse() {
                    Ok(value) => Some(value),
                    Err(_) => None,
                },
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
    pub selected_vm_idx: usize, // idx in table_state
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
            selected_vm_idx: 0,
            current_screen: Screen::List,
            exit: false,
        };
        state.refresh();
        Ok(state)
    }
    pub fn refresh(&mut self) {
        let kernels = files_in_directory(&format!("{}/kernels", &self.base_dir)).ok();
        let images = files_in_directory(&format!("{}/images", &self.base_dir)).ok();
        let mut vms = get_vms(&self.base_dir).map_or(vec![], |vms| vms);
        vms.sort_by(|vm1, vm2| vm1.name.cmp(&vm2.name));
        self.kernels = kernels;
        self.images = images;
        self.vms = vms;
    }
    pub fn start_stop_vm(&mut self) {
        if let Some(current_vm) = self.vms.get_mut(self.selected_vm_idx) {
            match current_vm.pid {
                Some(_) => {
                    match current_vm.kill() {
                        Ok(_) => {
                            current_vm.pid = None;
                            self.current_screen = Screen::List;
                            ()
                        }
                        Err(err) => {
                            self.current_screen = Screen::StartStop(StartStopState {
                                err_str: Some(err),
                                vertical_scroll_bar_state: ScrollbarState::default(),
                                vertical_scroll_bar_pos: 0,
                            })
                        }
                    };
                }
                None => {
                    match std::process::Command::new(format!("{}/startnb.sh", self.base_dir))
                        .args([
                            "-f",
                            &format!("{}/etc/{}.conf", self.base_dir, current_vm.name),
                            "-d",
                        ])
                        .current_dir(&self.base_dir)
                        .output()
                    {
                        Ok(res) => {
                            if res.stderr.is_empty() {
                                // Updating the VM info
                                current_vm.update_pid(&self.base_dir);
                                // Everything is fine, going back to the main screen
                                self.current_screen = Screen::List;
                            } else {
                                let err_str = String::from_utf8(res.stderr).unwrap();
                                let err_str_lines = err_str.lines().count();
                                self.current_screen = Screen::StartStop(StartStopState {
                                    err_str: Some(err_str),
                                    vertical_scroll_bar_state: ScrollbarState::default()
                                        .content_length(err_str_lines),
                                    vertical_scroll_bar_pos: 0,
                                })
                            }
                        }
                        Err(err) => {
                            println!("Err: {err}");
                        }
                    }
                    ()
                }
            }
        }
    }

    pub fn delete_vm(&mut self) {
        let current_vm = match self.vms.get_mut(self.selected_vm_idx) {
            Some(vm) => vm,
            None => return, // This should never happened (unless self.selected_vm_idx is incorrect)
        };
        current_vm.pid.map(|_| current_vm.kill());
        let file_to_delete = format!("{}/etc/{}.conf", self.base_dir, current_vm.name);
        std::fs::remove_file(&file_to_delete)
            .expect(&format!("Couldn't delete file {}", file_to_delete));
        self.vms.remove(self.selected_vm_idx);
        self.selected_vm_idx = self.selected_vm_idx.min(self.vms.len() - 1);
    }
}

fn get_vms(base_directory: &str) -> Result<Vec<Vm>, Box<dyn std::error::Error>> {
    let conf_directory = format!("{}/etc", &base_directory);
    let vm_confs = files_in_directory(&conf_directory)
        .ok()
        .map_or(vec![], |vm_confs| {
            vm_confs
                .iter()
                .filter(|vm_conf_file| {
                    // filename must ends with ".conf"
                    vm_conf_file
                        .file_name()
                        .to_str()
                        .unwrap_or("")
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
                            // We already checked that 'line' contains '=', so the call to unwrap() will always succeed
                            let (key, value) = line.trim().split_once('=').unwrap();
                            (key.to_owned(), value.to_owned())
                        })
                        .collect();
                    // If the hashmap doesn't contain the 'vm' key, we discard it
                    match hashmap.get("vm") {
                        Some(_) => Some((
                            hashmap,
                            vm_conf_file
                                .file_name()
                                .into_string()
                                .unwrap_or("".to_string()),
                        )),
                        None => None,
                    }
                })
                .map(|(config_data, vm_conf_file)| {
                    let vm_conf_file = vm_conf_file.strip_suffix(".conf").unwrap(); // We filtered the files ending with '.conf', so this unwrap() always succeed
                    let mut vm = Vm {
                        pid: None,
                        name: vm_conf_file.to_string(),
                        config_data: config_data,
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
