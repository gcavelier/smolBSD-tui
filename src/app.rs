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

#[derive(Default)]
pub struct Vm {
    ///
    /// Mandatory parameters
    ///

    /// Name of the config file without the '.conf' extension
    pub name: String,
    /// Disk image
    pub img: String,
    /// Kernel to load
    pub kernel: String,

    ///
    /// Optional parameters
    ///
    pub mem: String,
    pub cores: u8,
    pub hostfwd: String,
    pub editprotect: bool,
    pub rmprotect: bool,
    pub qmp_port: u16,
    pub bridgenet: String,
    pub share: String,
    pub extra: String,

    /// State
    pub pid: Option<i32>,
    pub cpu_usage: u8,
}

impl Vm {
    fn new(vm_conf: Vec<(&str, &str)>, conf_file: &DirEntry) -> Result<Self, String> {
        // Convert vm_conf into a hashmap to check if it contains all the mandatory keys
        let mut vm_conf_hashmap: HashMap<&str, &str> = HashMap::new();
        for (key, value) in &vm_conf {
            vm_conf_hashmap.insert(*key, *value);
        }

        // Check if all the mandatory keys are present
        if !vm_conf_hashmap.contains_key("img") {
            return Err("Missing mandatory parameter 'img'".into());
        }
        if !vm_conf_hashmap.contains_key("kernel") {
            return Err("Missing mandatory parameter 'kernel'".into());
        }

        let mut res = Vm {
            name: conf_file
                .file_name()
                .to_string_lossy()
                .strip_suffix(".conf")
                // SAFETY: this unwrap always succeed (get_vms() filtered the .conf files)
                .unwrap()
                .to_string(),
            ..Default::default()
        };

        for (key, value) in vm_conf {
            match key {
                "vm" => {}
                "img" => res.img = value.to_string(),
                "kernel" => res.kernel = value.to_string(),
                "mem" => res.mem = value.to_string(),
                "cores" => {
                    res.cores = value.parse().map_err(|err| {
                        format!("Failed to convert 'cores' parameter to a u8: {err}")
                    })?
                }
                "hostfwd" => res.hostfwd = value.to_string(),
                "editprotect" => res.editprotect = parse_bool(value)?,
                "rmprotect" => res.rmprotect = parse_bool(value)?,
                "qmp_port" => res.qmp_port = value.parse().unwrap_or(0),
                "bridgenet" => res.bridgenet = value.to_string(),
                "share" => res.share = value.to_string(),
                "extra" => res.extra = value.to_string(),
                _ => {}
            }
        }

        Ok(res)
    }

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

///
/// This function reads all the files in {base_directory}/etc/ and, for each file,
/// it will construct the corresponding Vm struct.
///
/// If a configuration file contains an unknown parameter or an invalid value,
/// the whole Vm struct will be discarded and the Vm won't be shown at all.
///
fn get_vms(base_directory: &str) -> Result<Vec<Vm>, Box<dyn std::error::Error>> {
    // This is the Vec we will return
    let mut vm_confs: Vec<Vm> = Vec::new();

    // We only want to get the .conf files
    let conf_files: Vec<_> = files_in_directory(&format!("{base_directory}/etc"))?
        .into_iter()
        .filter(|file| file.file_name().to_string_lossy().ends_with(".conf"))
        .collect();

    for conf_file in conf_files {
        let vm_conf_file_data = std::fs::read_to_string(conf_file.path())?;
        let vm_conf: Vec<(&str, &str)> = vm_conf_file_data
            .lines()
            .filter(|line| !line.starts_with('#') && line.contains('='))
            .map(|line| {
                // We already checked that 'line' contains '=', so calling unwrap() is ok
                let (key, value) = line.split_once('=').unwrap();
                (key, value)
            })
            .collect();

        match Vm::new(vm_conf, &conf_file) {
            Ok(mut vm) => {
                vm.update_pid(base_directory);
                vm_confs.push(vm)
            }
            //
            // TODO: indicate an "invalid" state showing the reason it is invalid ?
            // => This implies replacing the "RUNNING" column in the UI with a "STATE" column
            // for example
            //
            Err(err) => {
                println!("{}: {err}", conf_file.file_name().to_string_lossy());
                continue;
            }
        };
    }

    Ok(vm_confs)
}

fn files_in_directory(directory: &str) -> Result<Vec<DirEntry>, Box<dyn std::error::Error>> {
    let res: Vec<_> = std::fs::read_dir(directory)?
        .filter_map(|res_dir_entry| res_dir_entry.ok())
        .filter(|dir_entry| dir_entry.file_type().is_ok_and(|entry| entry.is_file()))
        .collect();
    Ok(res)
}

fn parse_bool(input: &str) -> Result<bool, String> {
    match input {
        "true" | "True" => Ok(true),
        "false" | "False" => Ok(false),
        _ => Err(format!("cannot convert '{input}' into a boolean")),
    }
}
