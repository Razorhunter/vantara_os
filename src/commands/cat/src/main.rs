use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::{Path};
use std::process::{exit};
use vantara::{package_name, print_version, expand_wildcards, safe_eprintln, safe_println};

struct Options {
    number: bool,           // --number (-n)
    number_nonblank: bool,  // --number-nonblank (-b)
    squeeze_blank: bool,    // --squeeze-blank (-s)
    show_ends: bool,        // --show-ends (-E)
    show_tabs: bool,        // --show-tabs (-T)
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut options = Options {
        number: false,
        number_nonblank: false,
        squeeze_blank: false,
        show_ends: false,
        show_tabs: false,
    };

    //Check for empty arguments
    if args.is_empty() {
        safe_println(format_args!("{}: no arguments", package_name!()));
        print_usage();
        exit(1);
    }

    let mut paths: Vec<String> = Vec::new();

    for arg in &args {
        match arg.as_str() {
            "--help" => { print_usage(); exit(0); },
            "--version" => { print_version!(); exit(0); },
            "--number" => options.number = true,
            "--number-nonblank" => options.number_nonblank = true,
            "--squeeze-blank" => options.squeeze_blank = true,
            "--show-ends" => options.show_ends = true,
            "--show-tabs" => options.show_tabs = true,
            _ if arg.starts_with('-') => {
                for c in arg.chars().skip(1) {
                    match c {
                        'n' => options.number = true,
                        'b' => options.number_nonblank = true,
                        's' => options.squeeze_blank = true,
                        'E' => options.show_ends = true,
                        'T' => options.show_tabs = true,
                        _ => {
                            safe_eprintln(format_args!("{}: unknown flag -{}", package_name!(), c));
                            exit(1);
                        }
                    }
                }
            },
            _ if !arg.starts_with('-') => paths.push(arg.clone()),
            _ => {
                safe_eprintln(format_args!("{}: unknown option '{}'", package_name!(), arg));
                exit(1);
            }
        }
    }

    if options.number_nonblank {
        options.number = false;
    }

    //check for empty path
    if paths.is_empty() {
        safe_println(format_args!("{}: please specify at least one (1) filename", package_name!()));
        print_usage();
        exit(1);
    }

    paths = expand_wildcards(&paths);

    for path in &paths {
        let src = Path::new(&path);

        //Checking source exist or not. if only one failed, skip command
        if !src.exists() {
            safe_println(format_args!("{}: cannot stat '{}': No such file or directory", package_name!(), src.display()));
            exit(1);
        }
    }

    if let Err(e) = read_files(paths, &options) {
        safe_eprintln(format_args!("{}: failed to read: {}", package_name!(), e));
        exit(1);
    }
}


fn read_files(paths: Vec<String>, options: &Options) -> io::Result<()> {
    let mut line_number = 1;

    for path in &paths {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut previous_blank = false;

        for line_result in reader.lines() {
            let mut line = line_result?;
            let is_blank = line.trim().is_empty();

            // --squeeze-blank
            if options.squeeze_blank && is_blank && previous_blank {
                continue;
            }
            previous_blank = is_blank;

            // --show-tabs
            if options.show_tabs {
                line = line.replace("\t", "^I");
            }

            // --show-ends
            if options.show_ends {
                line.push('$');
            }

            let prefix = if options.number_nonblank && !is_blank {
                let p = format!("{:>6}\t", line_number);
                line_number += 1;
                p
            } else if options.number && !options.number_nonblank {
                let p = format!("{:>6}\t", line_number);
                line_number += 1;
                p
            } else {
                String::new()
            };

            safe_println(format_args!("{}{}", prefix, line));
        }
    }

    Ok(())
}

fn print_usage() {
    safe_println(format_args!("Usage: {} -[OPTIONS] [FILE..]", package_name!()));
    safe_println(format_args!("     n | --number            Show number all lines"));
    safe_println(format_args!("     b | --number-nonblank   Number only non-blank lines"));
    safe_println(format_args!("     s | --squeeze-blank     Squeeze blank lines"));
    safe_println(format_args!("     E | --show-ends         Show $ at end of lines"));
    safe_println(format_args!("     T | --show-tabs         Show TAB characters as ^I"));
    safe_println(format_args!("     --help                  Show help"));
    safe_println(format_args!("     --version               Show version"));
}