mod rtar_utility;

use std::env;
use std::path::Path;
use anyhow::Result;
use vantara::{safe_eprintln, safe_println, package_name};

struct Options<'a> {
    create: bool,
    extract: bool,
    verbose: bool,
    compression: Option<&'a str>,
    archive_path: Option<&'a str>,
    change_dir: Option<&'a str>,
    inputs: Vec<&'a str>,
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        print_usage_and_exit();
    }

    let mut options = Options {
        create: false,
        extract: false,
        verbose: false,
        compression: None,
        archive_path: None,
        change_dir: None,
        inputs: Vec::new(),
    };

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];

        if arg.starts_with('-') {
            for ch in arg.chars().skip(1) {
                match ch {
                    'c' => options.create = true,
                    'x' => options.extract = true,
                    'v' => options.verbose = true,
                    'z' => options.compression = Some("gz"),
                    'j' => options.compression = Some("bz2"),
                    'J' => options.compression = Some("xz"),
                    'f' => {
                        i += 1;
                        options.archive_path = args.get(i).map(|s| s.as_str());
                    },
                    'C' => {
                        i += 1;
                        options.change_dir = args.get(i).map(|s| s.as_str());
                    }
                    _ => {
                        safe_eprintln(format_args!("{}: unknown flag -{}", package_name!(), ch));
                        print_usage_and_exit();
                    }
                }
            }
        } else {
            options.inputs.push(arg);
        }

        i += 1;
    }

    let archive = options.archive_path
        .map(Path::new)
        .expect("Please specify archive name by -f");

    if options.create {
        if options.inputs.is_empty() {
            safe_eprintln(format_args!("{}: no such file or directory", package_name!()));
            print_usage_and_exit();
        }
        rtar_utility::create_archive(archive, &options.inputs, options.compression, options.verbose)?;
    } else if options.extract {
        let target_dir = options.change_dir.map(Path::new).unwrap_or_else(|| Path::new("."));
        rtar_utility::extract_auto(archive, target_dir, options.verbose)?;
    } else {
        print_usage_and_exit();
    }

    Ok(())
}

fn print_usage_and_exit() -> ! {
    safe_println(format_args!(
        "Usage: tar -cf FILE.tar [INPUTS...]
     tar -czf FILE.tar.gz [INPUTS...]
     tar -xf FILE.tar [-C DIR]
     Flags:
         -c  create
         -x  extract
         -f  archive filename
         -z  gzip compression
         -j  bzip2 compression
         -J  xz compression
         -v  verbose
         -C  extract destination"
    ));
    std::process::exit(1);
}
