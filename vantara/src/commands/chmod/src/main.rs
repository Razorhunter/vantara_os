use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::env;
use std::process::exit;
use vantara::{safe_println, safe_eprintln, print_version, package_name, expand_wildcards};
use walkdir::WalkDir;

struct Options {
    recursive: bool,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut options = Options {
        recursive: false
    };

    //Check for empty arguments
    if args.is_empty() {
        safe_eprintln(format_args!("{}: no arguments", package_name!()));
        print_usage();
        exit(1);
    }

    let mut paths: Vec<String> = Vec::new();
    let mut mode = "";
    let mut arg_iter = args[1..].iter();

    while let Some(arg) = arg_iter.next() {
        if arg == "-R" {
            options.recursive = true;
        } else if arg == "--version" {
            print_version!();
            exit(0);
        } else if arg == "--help" {
            print_usage();
            exit(0);
        } else if mode.is_empty() {
            mode = arg;
        } else {
            paths.push(arg.clone());
        }
    }

    if paths.is_empty() {
        safe_println(format_args!("{}: please specify at least one (1) filename or directory", package_name!()));
        print_usage();
        exit(1);
    }

    paths = expand_wildcards(&paths);

    for path in &paths {
        let target_path = PathBuf::from(path);

        if options.recursive && target_path.is_dir() {
            for entry in WalkDir::new(&target_path) {
                if let Ok(entry) = entry {
                    apply_mode(mode, entry.path());
                }
            }
        } else {
            apply_mode(mode, &target_path);
        }
    }
}

fn apply_mode(mode_str: &str, path: &Path) {
    match fs::metadata(path) {
        Ok(metadata) => {
            let old_mode = metadata.permissions().mode();
            let new_mode = if let Ok(num) = u32::from_str_radix(mode_str, 8) {
                num
            } else {
                match apply_symbolic(mode_str, old_mode) {
                    Some(m) => m,
                    None => {
                        safe_eprintln(format_args!("{}: invalid symbolic mode: {}", package_name!(), mode_str));
                        return;
                    }
                }
            };

            let mut perms = metadata.permissions();
            perms.set_mode(new_mode);
            if let Err(e) = fs::set_permissions(path, perms) {
                safe_eprintln(format_args!("{}: failed to set permissions on {:?}: {}", package_name!(), path, e));
            }
        }
        Err(e) => safe_eprintln(format_args!("{}: annot access {:?}: {}", package_name!(), path, e)),
    }
}

fn apply_symbolic(symbolic: &str, current: u32) -> Option<u32> {
    let mut mode = current;

    for part in symbolic.split(',') {
        let part = if part.starts_with('+') || part.starts_with('-') || part.starts_with('=') {
            // Auto-apply 'a' if no who is specified
            format!("a{}", part)
        } else {
            part.to_string()
        };

        if part.contains('+') || part.contains('-') {
            let (who, rest) = part.split_at(1);
            let op = rest.chars().next()?;
            let perms = &rest[1..];

            let targets = match who {
                "u" => 0o700,
                "g" => 0o070,
                "o" => 0o007,
                "a" => 0o777,
                _ => return None,
            };

            for c in perms.chars() {
                let bit = match c {
                    'r' => 0o444,
                    'w' => 0o222,
                    'x' => 0o111,
                    _ => return None,
                };

                match op {
                    '+' => mode |= bit & targets,
                    '-' => mode &= !(bit & targets),
                    _ => return None,
                }
            }
        } else if let Some((who, perm)) = part.split_once('=') {
            let mut new_bits = 0;
            for c in perm.chars() {
                new_bits |= match c {
                    'r' => 0o4,
                    'w' => 0o2,
                    'x' => 0o1,
                    _ => return None,
                };
            }

            let shift = match who {
                "u" => 6,
                "g" => 3,
                "o" => 0,
                "a" => {
                    // Apply to all: clear & set for each
                    mode &= !0o777;
                    mode |= new_bits << 6;
                    mode |= new_bits << 3;
                    mode |= new_bits;
                    continue;
                }
                _ => return None,
            };

            mode &= !(0o7 << shift);
            mode |= new_bits << shift;
        } else {
            return None;
        }
    }

    Some(mode)
}

fn print_usage() {
    safe_println(format_args!("Usage: chmod -[OPTIONS] [MODE] [TARGET]"));
    safe_println(format_args!("     R           Recursive"));
    safe_println(format_args!("     --help      Show help"));
    safe_println(format_args!("     --version   Show version"));
}