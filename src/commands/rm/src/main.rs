use std::env;
use std::fs;
use std::io::{Write};
use std::path::Path;
use std::process::{exit};
use vantara::{package_name, expand_wildcards, safe_println, safe_eprintln, print_version, confirm};

struct Options {
    recursive: bool,
    force_ignore: bool,
    interactive: bool,
    trash_mode: bool
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut options = Options {
        recursive: false,
        force_ignore: false,
        interactive: true,
        trash_mode: false,
    };

    //Check for empty arguments
    if args.is_empty() {
        safe_println(format_args!("{}: no arguments", package_name!()));
        print_usage();
        exit(1);
    }

    let mut paths: Vec<String> = Vec::new();

    for arg in args {
        match arg.as_str() {
            "--non-interactive" => options.interactive = false,
            "--trash" => options.trash_mode = true,
            "--help" => { print_usage(); return; },
            "--version" => { print_version!(); return; },
            _ if arg.starts_with('-') => {
                for c in arg.chars().skip(1) {
                    match c {
                        'r' => options.recursive = true,
                        'f' => options.force_ignore = true,
                        _ => {
                            safe_eprintln(format_args!("{}: unknown flag -{}", package_name!(), c));
                            exit(1);
                        }
                    }
                }
            },
            _ if !arg.starts_with('-') => paths.push(arg.clone()),
            _ => {}
        }
    }

    paths = expand_wildcards(&paths);

    //check for empty path
    if paths.is_empty() {
        safe_println(format_args!("{}: please specify directory or filename", package_name!()));
        print_usage();
        exit(1);
    }

    if options.interactive {
        if paths.len() > 1 {
            safe_println(format_args!("{} items match your request", paths.len()));
            if !confirm(&format!("Remove all these files/directories {}?", if options.trash_mode { "to trash bin" } else { "permanently" })) {
                safe_println(format_args!("{}: aborted.", package_name!()));
                return;
            }
        } else {
            if !confirm(&format!("Remove '{}' {}?", &paths[0], if options.trash_mode { "to trash bin" } else { "permanently" })) {
                safe_println(format_args!("{}: skipped '{}'", package_name!(), paths[0]));
                return;
            }
        }
    }

    for path in paths {
        if path == "/" {
            safe_println(format_args!("{}: refused to remove.", package_name!()));
            exit(1);
        } else {
            if let Ok(metadata) = fs::symlink_metadata(&path) {
                let ftype = metadata.file_type();

                if options.trash_mode {
                    if let Err(e) = move_to_trash(&path) {
                        if !options.force_ignore {
                            safe_println(format_args!("{}: failed to move to trash '{}': {}", package_name!(), path, e));
                        }
                    }
                    continue;
                }

                if ftype.is_dir() && !ftype.is_symlink() { //Delete directory
                    if options.recursive {
                        if let Err(e) = remove_dir_recursive(&path) {
                            if !options.force_ignore {
                                safe_eprintln(format_args!("{}: failed to remove directory '{}': {}", package_name!(), path, e));
                            }
                        }
                    } else {
                        if !options.force_ignore {
                            safe_println(format_args!("{}: '{}' is a directory. Use -r to remove recursively.", package_name!(), path));
                        }
                    }
                } else { //Delete file
                    if let Err(e) = fs::remove_file(&path) {
                        if !options.force_ignore {
                            safe_eprintln(format_args!("{}: failed to remove file '{}': {}", package_name!(), path, e));
                        }
                    }
                }
            } else {
                if let Ok(meta) = fs::symlink_metadata(&path) {
                    if meta.file_type().is_symlink() {
                        // Confirm broken symlink
                        if let Err(e) = fs::remove_file(&path) {
                            if !options.force_ignore {
                                safe_eprintln(format_args!("{}: failed to remove broken symlink '{}': {}", package_name!(), path, e));
                            }
                        } else if options.interactive {
                            safe_println(format_args!("{}: removed broken symlink '{}'", package_name!(), path));
                        }
                    } else {
                        safe_println(format_args!("{}: '{}' does not exist", package_name!(), path));
                    }
                } else {
                    safe_println(format_args!("{}: '{}' does not exist", package_name!(), path));
                }
            }
        }
    }
}

fn remove_dir_recursive(path: &str) -> Result<(), String> {
    let entries = fs::read_dir(path).map_err(|e| format!("{}: read dir failed: {}", package_name!(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("{}: directory entry failed: {}", package_name!(), e))?;
        let path = entry.path();

        if path.is_dir() {
            remove_dir_recursive(path.to_str().unwrap())?;
        } else {
            fs::remove_file(path.to_str().unwrap()).map_err(|e| format!("{}: removed failed: {}", package_name!(), e))?;
        }
    }

    fs::remove_dir(path).map_err(|e| format!("{}: remove failed: {}", package_name!(), e))
}

fn move_to_trash(path_str: &str) -> Result<(), std::io::Error> {
    let path: &Path = Path::new(&path_str);
    let trash_dir = Path::new("/.trash");
    let index_path = trash_dir.join(".index");

    if !trash_dir.exists() {
        fs::create_dir_all(trash_dir)?;
    }

    let file_name = path.file_name().unwrap_or_else(|| std::ffi::OsStr::new("unnamed"));
    let mut new_path = trash_dir.join(file_name);

    //Avoid file_name clash in trash bin
    let mut counter = 1;
    while new_path.exists() {
        let mut new_name = file_name.to_os_string();
        new_name.push(format!(".{}", counter));
        new_path = trash_dir.join(new_name);
        counter += 1;
    }

    let binding = fs::canonicalize(path)?;
    let old_path_str = binding.parent().and_then(|p| p.to_str()).unwrap_or("/");
    fs::rename(path, &new_path)?;

    //Stored in index
    let mut index_file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(index_path)?;

    writeln!(index_file, "{}:{}", new_path.file_name().unwrap().to_str().unwrap(), old_path_str)?;

    Ok(())
}

fn print_usage() {
    safe_println(format_args!("Usage: {} -[OPTIONS] [FILES OR DIRECTORY...]", package_name!()));
    safe_println(format_args!("     r                   Remove directories and their contents recursively"));
    safe_println(format_args!("     f                   Ignore nonexistent files and arguments, and force remove"));
    safe_println(format_args!("     --trash             Move file or directory into trash bin to restore later"));
    safe_println(format_args!("     --non-interactive   Skipped remove confirmation message"));
    safe_println(format_args!("     --help              Show help"));
    safe_println(format_args!("     --version           Show version"));
}