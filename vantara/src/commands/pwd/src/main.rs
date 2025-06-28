use std::env;
use std::fs;
use vantara::{safe_eprintln, safe_println, print_version, package_name};
use std::process::exit;

#[derive(Default, Debug)]
struct Options {
    logical: bool,
    physical: bool,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut options = Options {
        logical: true,
        physical: false,
    };

    let mut arg_iter = args[1..].iter();

    while let Some(arg) = arg_iter.next() {
        if arg == "--version" {
            print_version!();
            exit(0);
        } else if arg == "--help" {
            print_usage();
            exit(0);
        } else if arg.starts_with('-') && arg.len() > 1 {
            for c in arg.chars().skip(1) {
                match c {
                    'L' => options.logical = true,
                    'P' => options.physical = true,
                    _ => {
                        safe_eprintln(format_args!("{}: unknown flag: -{}", package_name!(), c));
                        exit(1);
                    }
                }
            }
        }
    }

    if options.logical {
        match env::var("PWD") {
            Ok(pwd) => safe_println(format_args!("{}", pwd)),
            Err(_) => {
                match env::current_dir() {
                    Ok(path) => safe_println(format_args!("{}", path.display())),
                    Err(e) => safe_eprintln(format_args!("{}: error: {}", package_name!(), e)),
                }
            }
        }
    } else {
        match env::current_dir() {
            Ok(path) => {
                let real_path = fs::canonicalize(path);
                match real_path {
                    Ok(resolved) => safe_println(format_args!("{}", resolved.display())),
                    Err(e) => safe_eprintln(format_args!("{}: error resolving path: {}", package_name!(), e)),
                }
            }
            Err(e) => safe_eprintln(format_args!("{}: error: {}", package_name!(), e)),
        }
    }
}

fn print_usage() {
    safe_println(format_args!("Usage: {} -[OPTIONS] [TARGET]", package_name!()));
    safe_println(format_args!("     L           Show logical path (follow symbolic link)"));
    safe_println(format_args!("     P           Show physical path (resolv symbolic link)"));
    safe_println(format_args!("     --help      Show help"));
    safe_println(format_args!("     --version   Show version"));
}
