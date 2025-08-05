use std::env;
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{Write, BufReader, BufRead};
use std::path::Path;
use std::process::{exit};
use vantara::{package_name, safe_println, safe_eprintln, print_version, confirm};

const DEFAULT_TRASH_DIR: &str = "/.trash";
const DEFAULT_TRASH_INDEX_DIR: &str = "/.trash/.index";

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    for arg in args {
        match arg.as_str() {
            "-l" | "--list" => { show_trash_list(); return; },
            "-e" | "--empty" => { empty_trash(); return; },
            "-r"| "--restore" => { trash_restore_specific(); return; },
            "--restore-all" => { trash_restore_all(); return; },
            "--help" => { print_usage(); return; },
            "--version" => { print_version!(); return; }
            _ => {}
        }
    }
    
}

fn show_trash_list() {
    let file = match File::open(DEFAULT_TRASH_INDEX_DIR) {
        Ok(f) => f,
        Err(_) => {
            safe_println(format_args!("{}: trash bin is empty", package_name!()));
            return;
        }
    };

    let reader = BufReader::new(file);
    safe_println(format_args!("Files in trash:"));
    for line in reader.lines().flatten() {
        if let Some((file, path)) = line.split_once(':') {
            safe_println(format_args!("- {} (from {})", file, path));
        }
    }
}

fn empty_trash() {
    if !confirm("Are you sure to empty the trash bin? This action cannot be undone") {
        safe_println(format_args!("{}: aborted", package_name!()));
        return;
    }

    match fs::read_dir(DEFAULT_TRASH_DIR) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let _ = fs::remove_file(path);
                } else if path.is_dir() {
                    let _ = fs::remove_dir_all(path);
                }
            }

            safe_println(format_args!("{}: trash bin now empty", package_name!()));
        }
        Err(_) => {
            safe_println(format_args!("{}: trash bin already empty or inaccessible.", package_name!()));
        }
    }
}

fn trash_restore_all() {
    let file = match File::open(DEFAULT_TRASH_INDEX_DIR) {
        Ok(f) => f,
        Err(_) => {
            safe_eprintln(format_args!("{}: nothing to restore", package_name!()));
            return;
        }
    };

    let reader = BufReader::new(file);
    for line in reader.lines().flatten() {
        if let Some((file_name, orig_path)) = line.split_once(':') {
            let trash_path = format!("{}/{}", DEFAULT_TRASH_DIR, file_name);
            let dest_path = format!("{}/{}", orig_path, file_name);

            if fs::rename(&trash_path, &dest_path).is_err() {
                safe_println(format_args!("{}: failed to restore '{}'", package_name!(), file_name));
            } else {
                safe_println(format_args!("{}: restored '{}' to '{}'", package_name!(), file_name, dest_path));
            }
        }
    }

    //Empty index after restore all
    let _ = fs::remove_file(DEFAULT_TRASH_INDEX_DIR);
}

fn trash_restore_specific() {
    let file = match File::open(DEFAULT_TRASH_INDEX_DIR) {
        Ok(f) => f,
        Err(_) => {
            safe_eprintln(format_args!("{}: nothing to restore", package_name!()));
            return;
        }
    };

    let args: Vec<String> = env::args().skip(2).collect();
    let mut files: Vec<String> = Vec::new();

    for arg in args {
        files.push(arg.clone());
    }

    //Checking for empty path
    if files.is_empty() {
        safe_println(format_args!("{}: please specify directory or file name", package_name!()));
        print_usage();
        exit(1);
    }

    //Read .index as map
    let reader = BufReader::new(file);
    let mut index_map: HashMap<String, String> = HashMap::new();

    for line in reader.lines().flatten() {
        if let Some((filename, orig_path)) = line.split_once(':') {
            index_map.insert(filename.to_string(), orig_path.to_string());
        }
    }

    let mut restored: Vec<String> = Vec::new();

    for name in files {
        let trash_path = Path::new(DEFAULT_TRASH_DIR).join(&name);

        if !trash_path.exists() {
            safe_println(format_args!("{}: '{}' not found in trash bin.", package_name!(), &name));
            continue;
        }

        match index_map.get(&*name) {
            Some(orig_path) => {
                let restore_path = Path::new(orig_path).join(&name);
                match fs::rename(&trash_path, &restore_path) {
                    Ok(_) => {
                        safe_println(format_args!("{}: '{}' Restored to '{}'", package_name!(), &name, &restore_path.display()));
                        restored.push(name.to_string());
                    },
                    Err(e) => safe_eprintln(format_args!("{}: failed to restore '{}': {}", package_name!(), name, e)),
                }
            }
            None => safe_println(format_args!("{}: no original path recorded for '{}'", package_name!(), &name)),
        }
    }

    //Update /.trash/.index
    if !restored.is_empty() {
        let filtered: Vec<String> = index_map
            .into_iter()
            .filter(|(k, _)| !restored.contains(k))
            .map(|(k, v)| format!("{}:{}", k, v))
            .collect();

        let mut file = match OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(DEFAULT_TRASH_INDEX_DIR)
        {
            Ok(f) => f,
            Err(e) => {
                safe_eprintln(format_args!("{}: failed to update index: {}", package_name!(), e));
                return;
            }
        };

        for line in filtered {
            let _ = writeln!(file, "{}", line);
        }
    }
}

fn print_usage() {
    safe_println(format_args!("Usage: {} -[OPTIONS]", package_name!()));
    safe_println(format_args!("     l | --list                  List all files or directories in trash bin"));
    safe_println(format_args!("     e                           Empty trash bin (cannot be undone)"));
    safe_println(format_args!("     r | --restore [FILE..]      Restore one file or directory"));
    safe_println(format_args!("     --restore-all               Restore all files and directories to it's original location"));
    safe_println(format_args!("     --help                      Show help"));
    safe_println(format_args!("     --version                   Show version"));
}
