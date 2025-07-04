use std::env;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use flate2::Compression;
use flate2::write::GzEncoder;
use vantara::{safe_eprintln, safe_println, package_name, print_version};
use std::process::exit;

struct Config {
    keep: bool,
    force: bool,
    verbose: bool,
    level: u32,
    files: Vec<String>,
}

fn main() {
    let config = parse_args();

    for file in &config.files {
        if let Err(e) = compress_file(file, &config) {
            safe_eprintln(format_args!("{}: {}: {}", package_name!(), file, e));
        }
    }
}


fn parse_args() -> Config {
    let mut keep = false;
    let mut force = false;
    let mut verbose = false;
    let mut level = 6; // Default gzip level
    let mut files = Vec::new();

    let args: Vec<String> = env::args().skip(1).collect();

    for arg in &args {
        match arg.as_str() {
            "--help" => { print_usage(); exit(0); },
            "--version" => { print_version!(); exit(0); },
            _ if arg.starts_with('-') => {
                for c in arg.chars().skip(1) {
                    match c {
                        'k' => keep = true,
                        'f' => force = true,
                        'v' => verbose = true,
                        _ => {
                            safe_eprintln(format_args!("{}: unknown flag -{}", package_name!(), c));
                            exit(1);
                        }
                    }
                }

                if arg.len() == 2 && arg.chars().nth(1).unwrap().is_digit(10) {
                    level = arg[1..2].parse().unwrap_or(6);
                }
            },
            _ => files.push(arg.clone())
        }
    }

    if files.is_empty() {
        safe_eprintln(format_args!("{}: usage: gzip [-kfv1-9] <file>...", package_name!()));
        exit(1);
    }

    Config {
        keep,
        force,
        verbose,
        level,
        files,
    }
}

fn compress_file(path: &str, config: &Config) -> std::io::Result<()> {
    let input_path = Path::new(path);
    let output_path = input_path.with_extension("gz");

    if output_path.exists() && !config.force {
        safe_eprintln(format_args!("{}: {} already exists (use -f to overwrite)", package_name!(), output_path.display()));
        return Ok(());
    }

    let input_file = File::open(&input_path)?;
    let mut reader = BufReader::new(input_file);

    let output_file = File::create(&output_path)?;
    let writer = BufWriter::new(output_file);

    let mut encoder = GzEncoder::new(writer, Compression::new(config.level));
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;
    encoder.write_all(&buffer)?;
    encoder.finish()?;

    if config.verbose {
        safe_println(format_args!("{}: {} → {}", package_name!(), path, output_path.display()));
    }

    if !config.keep {
        fs::remove_file(&input_path)?;
    }

    Ok(())
}

fn print_usage() {

}
