use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{exit};
use vantara::{package_name, safe_eprintln, safe_println, print_version};

struct Options {
    recursive: bool,
    verbose: bool,
    mode: Option<u32>
}

fn main() {
    //Take arguments from CLI
    let args: Vec<String> = env::args().skip(1).collect();
    let mut options = Options {
        recursive: false,
        verbose: false,
        mode: None
    };

    //Check for empty arguments
    if args.is_empty() {
        safe_println(format_args!("{}: no arguments", package_name!()));
        print_usage();
        exit(1);
    }

    let mut paths: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--help" | "-h" => {
                print_usage();
                return;
            }
            "--version" => {
                print_version!();
                return;
            }
            _ if args[i].starts_with('-') => {
                for c in args[i].chars().skip(1) {
                    match c {
                        'p' => options.recursive = true,
                        'v' => options.verbose = true,
                        'm' => {
                            i += 1;
                            if i >= args.len() {
                                safe_println(format_args!("{}: -m value is needed", package_name!()));
                                exit(1);
                            }

                            match parse_mode(&args[1]) {
                                Ok(m) => options.mode = Some(m),
                                Err(e) => {
                                    safe_eprintln(format_args!("{}", e));
                                    exit(1);
                                }
                            }
                        },
                        _ => {
                            safe_eprintln(format_args!("{}: unknown flag -{}", package_name!(), c));
                            exit(1);
                        }
                    }
                }
            },
            _ if !args[i].starts_with('-') => paths.push(args[i].clone()),
            _ => {},
        }
        i += 1;
    }

    //Check path entry
    if paths.is_empty() {
        safe_println(format_args!("{}: please specify directory name", package_name!()));
        print_usage();
        exit(1);
    }

    for path in paths {
        let p = Path::new(&path);
        let result = if options.recursive {
            fs::create_dir_all(&p)
        } else {
            fs::create_dir(&p)
        };

        match result {
            Ok(_) => {
                if let Some(m) = options.mode {
                    if let Err(e) = fs::set_permissions(&p, fs::Permissions::from_mode(m)) {
                        safe_eprintln(format_args!("{}: failed to give permissions to '{}': {}", package_name!(), path, e));
                    }
                }
                if options.verbose {
                    safe_println(format_args!("{}: directory '{}' created", package_name!(), path));
                }
            }
            Err(e) => {
                safe_eprintln(format_args!("{}: failed to create '{}': {}", package_name!(), path, e));
                exit(1);
            }
        }
    }
}

fn print_usage() {
    safe_println(format_args!("Usage: {} -[OPTIONS] -[MODE] [DIRECTORY NAME]", package_name!()));
    safe_println(format_args!("     p           Create parent directory if not exist"));
    safe_println(format_args!("     v           Prompt message for every directory created"));
    safe_println(format_args!("     -m MODE     Set permission (e.g: 755, 700)"));
    safe_println(format_args!("     --help      Show help"));
    safe_println(format_args!("     --version   Show version"));
}

fn parse_mode(mode_str: &str) -> Result<u32, String> {
    u32::from_str_radix(mode_str, 8).map_err(|_| format!("Invalid mode: {}", mode_str))
}
