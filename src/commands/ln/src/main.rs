use vantara::{safe_println, safe_eprintln, package_name, print_version};
use std::env;
use std::process::exit;
use std::path::Path;
use std::fs;
use std::os::unix::fs as unix_fs;

struct Options {
    symbolic: bool,
    force: bool,
    verbose: bool
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        safe_println(format_args!("{}: no arguments", package_name!()));
        exit(1);
    }

    let mut options = Options {
        symbolic: false,
        force: false,
        verbose: false
    };

    let mut paths: Vec<String> = Vec::new();

    for arg in &args {
        match arg.as_str() {
            "--version" => { print_version!(); exit(0); },
            "--help" => { print_usage(); exit(0); },
            _ if arg.starts_with('-') => {
                for c in arg.chars().skip(1) {
                    match c {
                        's' => options.symbolic = true,
                        'f' => options.force = true,
                        'v' => options.verbose = true,
                        _ => {
                            safe_eprintln(format_args!("{}: unknown flag -{}", package_name!(), c));
                            exit(1);
                        }
                    }
                }
            },
            _ => paths.push(arg.clone()),
        }
    }

    if paths.is_empty() || paths.len() != 2 {
        safe_println(format_args!("{}: please specify source and destination", package_name!()));
        exit(1);
    }

    let src_path = Path::new(&paths[0]);
    let dest_path = Path::new(&paths[1]);

    if dest_path.exists() {
        if !options.force {
            safe_println(format_args!("{}: unable to create symlink. filename exist at '{}'", package_name!(), dest_path.display()));
            exit(1);
        }

        fs::remove_file(dest_path)?;
    }

    if !src_path.exists() {
        safe_eprintln(format_args!("{}: cannot stat '{}': No such file or directory", package_name!(), src_path.display()));
        exit(1);
    }

    if options.symbolic {
        unix_fs::symlink(paths[0].clone(), dest_path)?;
    } else {
        fs::hard_link(paths[0].clone(), dest_path)?;
    }

    if options.verbose {
        safe_println(format_args!("{}: '{}' {} '{}", package_name!(), paths[1], if options.symbolic { "->" } else { "=> "}, paths[0]));
    }

    Ok(())
}

fn print_usage() {
    safe_println(format_args!("Usage: {} -[OPTIONS] [SOURCE] [DESTINATION]", package_name!()));
    safe_println(format_args!("     s           Symbolic link"));
    safe_println(format_args!("     f           Force replace symlink if exists"));
    safe_println(format_args!("     v           Show verbose output"));
    safe_println(format_args!("     --help      Show help"));
    safe_println(format_args!("     --version   Show version"));
}
