use regex::RegexBuilder;
use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use walkdir::WalkDir;
use vantara::{safe_println, safe_eprintln, package_name, print_version};
use std::process::exit;

struct Options {
    pattern: String,
    path: Option<String>,
    ignore_case: bool,
    invert: bool,
    line_number: bool,
    recursive: bool,
    list_files: bool,
    color: bool,
}

fn main() {
    let options = parse_args();

    let regex = RegexBuilder::new(&options.pattern)
        .case_insensitive(options.ignore_case)
        .build()
        .unwrap();

    let path_str = options.path.as_deref().unwrap_or("-");
    let target_path = Path::new(path_str);

    // stdin mode
    if path_str == "-" || !target_path.exists() || !target_path.is_file() && !target_path.is_dir() {
        let stdin = io::stdin();

        for (i, line) in stdin.lock().lines().enumerate() {
            let line = line.unwrap();
            let is_match = regex.is_match(&line);

            if is_match ^ options.invert {
                let mut output = String::new();
                if options.line_number {
                    output.push_str(&format!("{}:", i + 1));
                }

                if options.color {
                    let highlighted = regex.replace_all(&line, "\x1b[31m$0\x1b[0m").to_string();
                    output.push_str(&highlighted);
                } else {
                    output.push_str(&line);
                }

                println!("{}", output);
            }
        }
    } else {
        let paths: Box<dyn Iterator<Item = _>> = if options.recursive && target_path.is_dir() {
            Box::new(WalkDir::new(target_path)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_file())
                .map(|e| e.path().to_path_buf()))
        } else {
            Box::new(std::iter::once(target_path.to_path_buf()))
        };

        for path in paths {
            grep_input(&path, &options, &regex);
        }
    }
}

fn parse_args() -> Options {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        safe_println(format_args!("{}: no arguments", package_name!()));
        print_usage();
        exit(1);
    }

    let mut options = Options {
        pattern: String::new(),
        path: None,
        ignore_case: false,
        invert: false,
        line_number: false,
        recursive: false,
        list_files: false,
        color: false,
    };

    for arg in &args {
        match arg.as_str() {
            "--help" => { print_usage(); exit(0); },
            "--version" => { print_version!(); exit(0); },
            "--color" => options.color = true,
            _ if arg.starts_with('-') => {
                for c in arg.chars().skip(1) {
                    match c {
                        'i' => options.ignore_case = true,
                        'v' => options.invert = true,
                        'n' => options.line_number = true,
                        'r' => options.recursive = true,
                        'l' => options.list_files = true,
                        _ => {
                            safe_eprintln(format_args!("{}: unknown flag -{}", package_name!(), c));
                            exit(1);
                        }
                    }
                }
            }
            _ => {
                if options.pattern.is_empty() {
                    options.pattern = arg.clone();
                } else if options.path.is_none() {
                    options.path = Some(arg.clone());
                }
            }
        }
    }

    if options.pattern.is_empty() {
        safe_eprintln(format_args!("{}: pattern is required", package_name!()));
        exit(1);
    }

    if options.path.is_none() {
        options.path = Some("-".to_string());
    }

    options
}

fn grep_input(path: &Path, options: &Options, regex: &regex::Regex) {
    if let Ok(file) = File::open(path) {
        let reader = io::BufReader::new(file);
        let mut matched = false;

        for (i, line) in reader.lines().enumerate() {
            let line = line.unwrap();
            let is_matched = regex.is_match(&line);

            if is_matched ^ options.invert {
                matched = true;

                if options.list_files {
                    safe_println(format_args!("{}", path.display()));
                    return;
                }

                let mut output = String::new();

                if options.line_number {
                    output.push_str(&format!("{}:", i + 1));
                }

                if options.color {
                    let highlighted = regex.replace_all(&line, "\x1b[31m$0\x1b[0m").to_string();
                    output.push_str(&highlighted);
                } else {
                    output.push_str(&line);
                }

                safe_println(format_args!("{} <- {}", path.display(), output));
            }
        }

        if options.list_files && matched {
            safe_println(format_args!("{}", path.display()));
        }
    }
}

fn print_usage() {
    safe_println(format_args!(
        "Usage: {} [options] <pattern> [file|dir|-]\n\
         Options:\n\
         \t-i\t\tIgnore case\n\
         \t-v\t\tInvert match\n\
         \t-n\t\tShow line numbers\n\
         \t-r\t\tRecursive search in directories\n\
         \t-l\t\tList matching files only\n\
         \t--color\t\tHighlight match in red\n\
         \t--help\t\tShow help\n\
         \t--version\tShow version",
        package_name!()
    ));
}
