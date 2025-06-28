use std::env;
use std::fs;
use std::process::{exit};
use vantara::{package_name, expand_wildcards, safe_println, safe_eprintln, print_version, confirm};

struct Options {
    interactive: bool
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut options = Options {
        interactive: true,
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
            "--help" => { print_usage(); return; },
            "--version" => { print_version!(); return; },
            _ if !arg.starts_with('-') => paths.push(arg.clone()),
            _ => {}
        }
    }

    paths = expand_wildcards(&paths);

    //check for empty path
    if paths.is_empty() {
        safe_println(format_args!("{}: please specify directory name", package_name!()));
        print_usage();
        exit(1);
    }

    if options.interactive {
        if paths.len() > 1 {
            safe_println(format_args!("{} items match your request", paths.len()));
            if !confirm(&format!("Remove all these directories?")) {
                safe_println(format_args!("{}: aborted.", package_name!()));
                return;
            }
        } else {
            if !confirm(&format!("Remove '{}'?", &paths[0])) {
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
            if let Ok(metadata) = fs::metadata(&path) {
                if metadata.is_dir() {
                    remove_dir(&path);
                } else {
                    safe_eprintln(format_args!("{}: is not a directory", package_name!()));
                }
            } else {
                safe_println(format_args!("{}: '{}' does not exist", package_name!(), path));
            }
        }
    }
}

fn remove_dir(path: &str) {
    match fs::read_dir(path) {
        Ok(mut entries) => {
            if entries.next().is_some() {
                safe_eprintln(format_args!("{}: faile to remove '{}'. directory is not empty", package_name!(), path));
                exit(1);
            }

            if let Err(e) = fs::remove_dir(path) {
                safe_eprintln(format_args!("{}: failed to remove '{}': {}", package_name!(), path, e))
            };
        },
        Err(e) => safe_eprintln(format_args!("{}: failed to read '{}': {}", package_name!(), path, e))
    }
}

fn print_usage() {
    safe_println(format_args!("Usage: {} [OPTIONS] [DIRECTORY..]", package_name!()));
    safe_println(format_args!("     --non-interactive   Skipped remove confirmation message"));
    safe_println(format_args!("     --help              Show help"));
    safe_println(format_args!("     --version           Show version"));
}