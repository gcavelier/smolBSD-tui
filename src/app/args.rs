use std::path::Path;

fn show_help() {
    println!("\nThis program needs one argument, the path to a directory containing :");
    println!(" - the startnb.sh script");
    println!(" - the VMs configurations (etc/)");
    println!(" - the VMs kernels (kernels/) (Optionnal)");
    println!(" - the VMs images (images/) (Optionnal)\n");
}

pub fn get_base_dir() -> Option<String> {
    let mut args_iter = std::env::args();
    if args_iter.len() != 2 {
        show_help();
        return None;
    }

    // We only care about the first argument
    // The first element in args_iter is the program name,
    // so we get the second element (nth(1)) in args_iter
    if let Some(base_dir) = args_iter.nth(1) {
        // Handle the "help" cases
        if base_dir == "-h" || base_dir == "--help" {
            show_help();
            return None;
        }

        if !Path::new(&base_dir).is_dir() {
            println!("'{base_dir}' is not a directory");
            return None;
        }

        if !Path::new(&format!("{base_dir}/startnb.sh")).is_file() {
            println!("Couldn't find the startnb.sh script in '{base_dir}'");
            return None;
        } else if !Path::new(&format!("{base_dir}/etc/")).is_dir() {
            println!("Couldn't find a 'etc/' directory in '{base_dir}'");
            return None;
        }
        Some(
            Path::new(&base_dir)
                .canonicalize()
                .expect(&format!("Failed to canonicalize {base_dir}"))
                .to_str()
                .expect(&format!(
                    "Failed to convert canonicalized path ({base_dir}) to a String"
                ))
                .to_string()
                + "/",
        )
    } else {
        unreachable!(); // Because we already checked that args_iter.len() == 2
    }
}
