use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Seek, SeekFrom};
use std::env;
use vantara::{safe_eprintln, safe_println, print_version, package_name, expand_wildcards};
use std::process::exit;

struct Options {
    follow: bool,
    lines: Option<String>
}

fn main() -> io::Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl+C handler");

    let mut args = env::args().skip(1).peekable();
    let mut options = Options {
        follow: false,
        lines: None
    };

    let mut paths: Vec<String> = Vec::new();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" => { print_usage(); exit(0); },
            "--version" => { print_version!(); exit(0); },
            _ if arg.starts_with('-') => {
                for c in arg.chars().skip(1) {
                    match c {
                        'n' => {
                            if let Some(value) = args.next() {
                                options.lines = Some(value);
                            } else {
                                safe_eprintln(format_args!("{}: options -n requires an argument", package_name!()));
                                exit(1);
                            }
                        },
                        'f' => options.follow = true,
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

    if paths.is_empty() {
        safe_eprintln(format_args!("{}: please specify at least one (1) filename", package_name!()));
        print_usage();
        exit(1);
    }

    paths = expand_wildcards(&paths);

    let multiple = paths.len() > 1;

    for (i, path) in paths.iter().enumerate() {
        let mut file = File::open(&path)?;
        let reader = BufReader::new(file.try_clone()?);
        let all_lines: Vec<String> = reader.lines().filter_map(Result::ok).collect();
        let num_lines = options.lines
            .as_ref()
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(10);

        let start = if all_lines.len() >num_lines {
            all_lines.len() - num_lines
        } else {
            0
        };

        if multiple {
            if i > 0 {
                println!();
            }
            println!("==> {} <==", path);
        }

        for line in &all_lines[start..] {
            println!("{}", line);
        }

        if options.follow {
            let mut pos = file.metadata()?.len();
            while running.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_secs(1));
                file = File::open(&path)?;
                file.seek(SeekFrom::Start(pos))?;
                let reader = BufReader::new(file.try_clone()?);

                for line in reader.lines() {
                    if let Ok(l) = line {
                        println!("{}", l);
                    }
                }
                pos = file.metadata()?.len();
            }
        }
    }

    Ok(())
}

fn print_usage() {
    safe_println(format_args!("Usage: {} -[OPTIONS] [FILENAME..]", package_name!()));
    safe_println(format_args!("     n                   Show n number of lines"));
    safe_println(format_args!("     f                   Follow file"));
    safe_println(format_args!("     --help              Show help"));
    safe_println(format_args!("     --version           Show version"));
}
