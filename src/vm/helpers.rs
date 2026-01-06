use std::{fs::DirEntry, path::PathBuf};

use crate::vm::Vm;

///
/// This function reads all the files in {base_directory}/etc/ and, for each file,
/// it will construct the corresponding Vm struct.
///
pub fn get_vms(base_directory: &str) -> Result<Vec<Vm>, Box<dyn std::error::Error>> {
    // This is the Vec we will return
    let mut vm_confs: Vec<Vm> = Vec::new();

    // We only want to get the .conf files
    let conf_files: Vec<_> = files_in_directory(&format!("{base_directory}/etc"))?
        .into_iter()
        .filter(|file| file.file_name().to_string_lossy().ends_with(".conf"))
        .collect();

    for conf_file in conf_files {
        let vm = vm_from_conf(conf_file.path(), base_directory)?;
        vm_confs.push(vm);
    }

    Ok(vm_confs)
}

pub fn vm_from_conf(
    conf_file: PathBuf,
    base_directory: &str,
) -> Result<Vm, Box<dyn std::error::Error>> {
    let vm_conf_file_data = std::fs::read_to_string(&conf_file)?;
    let vm_conf: Vec<(&str, &str)> = vm_conf_file_data
        .lines()
        .filter(|line| !line.starts_with('#') && line.contains('='))
        .map(|line| {
            // We already checked that 'line' contains '=', so calling unwrap() is ok
            let (key, value) = line.split_once('=').unwrap();
            (key, value)
        })
        .collect();

    let mut vm = Vm::new(vm_conf, &conf_file);
    vm.update_state(base_directory);
    Ok(vm)
}

pub fn files_in_directory(directory: &str) -> Result<Vec<DirEntry>, Box<dyn std::error::Error>> {
    let res: Vec<_> = std::fs::read_dir(directory)?
        .filter_map(|res_dir_entry| res_dir_entry.ok())
        .filter(|dir_entry| dir_entry.file_type().is_ok_and(|entry| entry.is_file()))
        .collect();
    Ok(res)
}

pub fn parse_bool(input: &str) -> Result<bool, String> {
    match input.trim_matches('"') {
        "true" | "True" | "y" | "Y" | "yes" | "Yes" => Ok(true),
        "false" | "False" | "n" | "N" | "no" | "No" => Ok(false),
        _ => Err(format!("cannot convert '{input}' into a boolean")),
    }
}
