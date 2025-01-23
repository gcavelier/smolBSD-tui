use ratatui::widgets::TableState;
use std::{
    collections::HashMap,
    fs::{read_to_string, DirEntry},
    path::Path,
};

pub enum CurrentScreen {
    List,
    StartStop,
}

#[derive(Debug)]
pub struct Vm {
    pub name: String, // Name of the config file
    config_data: HashMap<String, String>,
    pub pid: Option<i32>,
    pub cpu_usage: u8,
}
pub struct State {
    base_dir: String,
    pub vms: Vec<Vm>,
    pub kernels: Option<Vec<DirEntry>>,
    pub images: Option<Vec<DirEntry>>,
    pub table_state: TableState,
    pub selected_vm_idx: usize, // idx in table_state
    pub current_screen: CurrentScreen,
    pub exit: bool,
}

impl State {
    pub fn new(base_dir: String) -> Result<Self, Box<dyn std::error::Error>> {
        let kernels = files_in_directory(&format!("{}/kernels", &base_dir)).ok();
        let images = files_in_directory(&format!("{}/images", &base_dir)).ok();
        let vms = get_vms(&format!("{}/etc", &base_dir)).map_or(vec![], |vms| vms);
        Ok(State {
            base_dir,
            vms,
            kernels,
            images,
            table_state: TableState::default(),
            selected_vm_idx: 0,
            current_screen: CurrentScreen::List,
            exit: false,
        })
    }
    pub fn start_stop_vm(&mut self) {
        if let Some(current_vm) = self.vms.get(self.selected_vm_idx) {
            // match current_vm.running {
            //     true => {kill(pid, sig)}
            //     false => std::process::Command::new(format!("{}/startnb.sh", self.base_dir))
            //         .args(["-f", "", "-d"]),
            // }
        }
    }
}

fn get_vms(directory: &str) -> Result<Vec<Vm>, Box<dyn std::error::Error>> {
    let vm_confs = files_in_directory(directory)
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
                    let pid_file = format!(
                        "{directory}/../qemu-{}.pid",
                        vm_conf_file.strip_suffix(".conf").unwrap() // We filtered the files ending with '.conf', so this unwrap() always succeed
                    );
                    Vm {
                        pid: match Path::new(&pid_file).exists() {
                            false => None,
                            true => match read_to_string(pid_file) {
                                Ok(res) => match res.trim().parse() {
                                    Ok(value) => Some(value),
                                    Err(_) => None,
                                },
                                Err(_) => None,
                            },
                        },
                        name: vm_conf_file,
                        config_data: config_data,
                        cpu_usage: 0,
                    }
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
