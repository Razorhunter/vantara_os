use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

struct Config {
    num_lines: usize,
    num_bytes: Option<usize>,
    quiet: bool,
    verbose: bool,
    files: Vec<String>,
}

fn parse_args() -> Config {
    let args: Vec<String> = env::args().skip(1).collect();

    let mut num_lines = 10;
    let mut num_bytes = None;
    let mut quiet = false;
    let mut verbose = false;
    let mut files = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-n" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("head: option requires an argument -- 'n'");
                    std::process::exit(1);
                }
                num_lines = args[i].parse().unwrap_or_else(|_| {
                    eprintln!("head: invalid number of lines: '{}'", args[i]);
                    std::process::exit(1);
                });
            }
            "-c" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("head: option requires an argument -- 'c'");
                    std::process::exit(1);
                }
                num_bytes = Some(args[i].parse().unwrap_or_else(|_| {
                    eprintln!("head: invalid number of bytes: '{}'", args[i]);
                    std::process::exit(1);
                }));
            }
            "-q" => quiet = true,
            "-v" => verbose = true,
            _ if args[i].starts_with('-') => {
                eprintln!("head: invalid option -- '{}'", args[i]);
                std::process::exit(1);
            }
            _ => files.push(args[i].clone()),
        }
        i += 1;
    }

    if files.is_empty() {
        files.push("-".to_string());
    }

    Config {
        num_lines,
        num_bytes,
        quiet,
        verbose,
        files,
    }
}

fn main() {
    let config = parse_args();
    let multiple = config.files.len() > 1;

    for (i, file) in config.files.iter().enumerate() {
        let input: Box<dyn Read> = if file == "-" {
            Box::new(io::stdin())
        } else {
            Box::new(File::open(file).unwrap_or_else(|_| {
                eprintln!("head: cannot open '{}' for reading", file);
                std::process::exit(1);
            }))
        };

        if config.verbose || (multiple && !config.quiet) {
            if i > 0 {
                println!();
            }
            println!("==> {} <==", file);
        }

        if let Some(n_bytes) = config.num_bytes {
            print_bytes(input, n_bytes);
        } else {
            print_lines(input, config.num_lines);
        }
    }
}

fn print_lines<R: Read>(reader: R, count: usize) {
    let buffered = BufReader::new(reader);
    for (i, line) in buffered.lines().enumerate() {
        if i >= count {
            break;
        }
        if let Ok(l) = line {
            println!("{}", l);
        }
    }
}

fn print_bytes<R: Read>(mut reader: R, count: usize) {
    let mut buffer = vec![0; count];
    if let Ok(n) = reader.read(&mut buffer) {
        print!("{}", String::from_utf8_lossy(&buffer[..n]));
    }
}
