use std::env;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;
use flate2::read::GzDecoder;
use vantara::{safe_eprintln, safe_println, package_name, print_version};
use std::process::exit;

struct Config {
    keep: bool,
    force: bool,
    verbose: bool,
    files: Vec<String>,
}

fn main() {
    let config = parse_args();

    for file in &config.files {
        if let Err(e) = decompress_file(file, &config) {
            safe_eprintln(format_args!("{}: {}: {}", package_name!(), file, e));
        }
    }
}

fn parse_args() -> Config {
    let mut keep = false;
    let mut force = false;
    let mut verbose = false;
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
            },
            _ => files.push(arg.clone())
        }
    }

    if files.is_empty() {
        safe_eprintln(format_args!("{}: usage: gunzip [-kfv] <file.gz>...", package_name!()));
        exit(1);
    }

    Config {
        keep,
        force,
        verbose,
        files,
    }
}

fn decompress_file(path: &str, config: &Config) -> std::io::Result<()> {
    let input_path = Path::new(path);

    if !input_path.extension().map_or(false, |ext| ext == "gz") {
        safe_eprintln(format_args!("{}: {}: not a .gz file", package_name!(), path));
        return Ok(());
    }

    let output_path = input_path.with_extension(""); // remove .gz
    if output_path.exists() && !config.force {
        safe_eprintln(format_args!("{}: {} already exists (use -f to overwrite)", package_name!(), output_path.display()));
        return Ok(());
    }

    let input_file = File::open(&input_path)?;
    let mut decoder = GzDecoder::new(BufReader::new(input_file));
    let output_file = File::create(&output_path)?;
    let mut writer = BufWriter::new(output_file);

    let mut buffer = Vec::new();
    decoder.read_to_end(&mut buffer)?;
    writer.write_all(&buffer)?;

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
