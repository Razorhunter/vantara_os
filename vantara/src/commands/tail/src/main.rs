use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Seek, SeekFrom};
use std::thread;
use std::time::Duration;

fn parse_args() -> (usize, bool, String) {
    let args: Vec<String> = env::args().collect();
    let mut lines: usize = 10;
    let mut follow = false;
    let mut file_path = String::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-n" | "--lines" => {
                if i + 1 < args.len() {
                    lines = args[i + 1].parse().unwrap_or(10);
                    i += 1;
                }
            }
            "-f" | "--follow" => {
                follow = true;
            }
            "-h" | "--help" => {
                print_help_and_exit();
            }
            "--version" => {
                println!("tail-rs 0.1.0");
                std::process::exit(0);
            }
            _ => {
                if args[i].starts_with("-") {
                    eprintln!("Unknown option: {}", args[i]);
                    print_help_and_exit();
                } else {
                    file_path = args[i].clone();
                }
            }
        }
        i += 1;
    }

    if file_path.is_empty() {
        eprintln!("Error: no input file specified.");
        print_help_and_exit();
    }

    (lines, follow, file_path)
}

fn print_help_and_exit() {
    println!(
        "Usage: tail [OPTION]... FILE\n\
        Print the last 10 lines of FILE to standard output.\n\n\
        Options:\n\
        -n, --lines NUM     Output the last NUM lines (default 10)\n\
        -f, --follow        Output appended data as the file grows\n\
        -h, --help          Display this help and exit\n\
        --version           Output version info and exit"
    );
    std::process::exit(0);
}

fn main() -> io::Result<()> {
    let (num_lines, follow, file_path) = parse_args();

    let mut file = File::open(&file_path)?;
    let mut reader = BufReader::new(file.try_clone()?);

    let lines: Vec<String> = reader.lines().filter_map(Result::ok).collect();

    let start = if lines.len() > num_lines {
        lines.len() - num_lines
    } else {
        0
    };

    for line in &lines[start..] {
        println!("{}", line);
    }

    if follow {
        let mut pos = file.metadata()?.len();
        loop {
            thread::sleep(Duration::from_secs(1));
            file = File::open(&file_path)?;
            file.seek(SeekFrom::Start(pos))?;
            let mut reader = BufReader::new(file.try_clone()?);

            for line in reader.lines() {
                if let Ok(l) = line {
                    println!("{}", l);
                }
            }
            pos = file.metadata()?.len();
        }
    }

    Ok(())
}
