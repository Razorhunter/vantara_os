use std::env;
use chrono::{Local, NaiveDateTime, TimeZone};
use std::fs::{self, OpenOptions};
use std::io;
use std::path::Path;
use std::time::{UNIX_EPOCH, SystemTime};
use filetime::{set_file_times, FileTime};
use vantara::{package_name, safe_println, safe_eprintln, print_version};
use std::process::{exit};

struct Options {
    change_access_time_only: bool,
    change_modification_time_only: bool,
    no_create: bool,
    set_time: Option<String>,
    specific_time_format: Option<String>,
    use_other_file_time: Option<String>,
}

fn main() -> io::Result<()> {
    let mut args = env::args().skip(1).peekable();

    let mut paths: Vec<String> = Vec::new();
    let mut options = Options {
        change_access_time_only: false,
        change_modification_time_only: false,
        no_create: false,
        set_time: None,
        specific_time_format: None,
        use_other_file_time: None,
    };

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" => { print_usage(); exit(0); },
            "--version" => { print_version!(); exit(0); },
            "--no-create" => options.no_create = true,
            _ if arg.starts_with('-') => {
                for c in arg.chars().skip(1) {
                    match c {
                        'a' => options.change_access_time_only = true,
                        'm' => options.change_modification_time_only= true,
                        'c' => options.no_create = true,
                        't' => {
                            if let Some(value) = args.next() {
                                options.set_time = Some(value);
                            } else {
                                safe_println(format_args!("{}: option -t requires an argument", package_name!()));
                                exit(1);
                            }
                        },
                        'd' => {
                            let part1 = args.next(); // Option<String>

                            if let Some(date) = part1.clone() {
                                if let Some(time) = args.peek().cloned() {
                                    if time.contains(':') {
                                        args.next(); // consume peeked
                                        options.specific_time_format = Some(format!("{} {}", date, time));
                                    } else {
                                        options.specific_time_format = Some(date);
                                    }
                                } else {
                                    options.specific_time_format = Some(date);
                                }
                            } else {
                                safe_println(format_args!("{}: option '-d' requires a date/time string", package_name!()));
                                exit(1);
                            }
                        },
                        'r' => {
                            if let Some(value) = args.next() {
                                options.use_other_file_time = Some(value);
                            } else {
                                safe_eprintln(format_args!("{}: option -r requires an argument", package_name!()));
                                exit(1);
                            }
                        },
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

    //Check for empty paths
    if paths.is_empty() {
        safe_eprintln(format_args!("{}: please specify at least one (1) filename", package_name!()));
        print_usage();
        exit(1);
    }

    //Set time
    let (atime, mtime) = if let Some(ref t) = options.set_time {
        match parse_dash_t(t) {
            Some(ts) => {
                let ft = FileTime::from_system_time(ts);
                (ft, ft)
            }
            None => {
                safe_println(format_args!("{}: failed to parse time -t '{}'", package_name!(), t));
                exit(1);
            }
        }
    } else if let Some(ref d) = options.specific_time_format {
        match parse_dash_d(d) {
            Some(ts) => {
                let ft = FileTime::from_system_time(ts);
                (ft, ft)
            }
            None => {
                safe_println(format_args!("{}: failed to parse time -d '{}'", package_name!(), d));
                exit(1);
            }
        }
    } else if let Some(ref file) = options.use_other_file_time {
        let meta = fs::metadata(file)?;
        let atime = FileTime::from_last_access_time(&meta);
        let mtime = FileTime::from_last_modification_time(&meta);
        (atime, mtime)
    } else {
        let now = FileTime::from_system_time(SystemTime::now());
        (now, now)
    };

    //Loop paths
    for filename in &paths {
        let path = Path::new(&filename);

        if path.exists() {
            let now = FileTime::from_system_time(SystemTime::now());
            filetime::set_file_mtime(path, now)?;
        } else {
            if options.no_create {
                continue;
            } else {
                OpenOptions::new()
                .create(true)
                .write(true)
                .open(path)?;
            }
        }

        let metadata = fs::metadata(path)?;
        let current_atime = FileTime::from_last_access_time(&metadata);
        let current_mtime = FileTime::from_last_modification_time(&metadata);

        let final_atime = if options.change_modification_time_only {
            current_atime
        } else {
            atime
        };

        let final_mtime = if options.change_access_time_only {
            current_mtime
        } else {
            mtime
        };

        set_file_times(path, final_atime, final_mtime)?;
    }

    Ok(())
}

fn parse_dash_d(input: &str) -> Option<SystemTime> {
    NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S")
        .or_else(|_| NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M"))
        .ok()
        .and_then(|naive| {
            let dt = Local.from_local_datetime(&naive).unwrap();
            Some(UNIX_EPOCH + std::time::Duration::from_secs(dt.timestamp() as u64))
        })
}

fn parse_dash_t(input: &str) -> Option<SystemTime> {
    let (date_str, sec) = if let Some(idx) = input.find('.') {
        (&input[..idx], &input[idx + 1..])
    } else {
        (input, "00")
    };

    let fmt = match date_str.len() {
        12 => "%Y%m%d%H%M",
        10 => "%y%m%d%H%M",
        _ => return None,
    };

    let full_input = format!("{}{}", date_str, sec);
    let full_fmt = format!("{}{}", fmt, "%S");

    NaiveDateTime::parse_from_str(&full_input, &full_fmt)
        .ok()
        .and_then(|naive| {
            let dt = Local.from_local_datetime(&naive).unwrap();
            Some(UNIX_EPOCH + std::time::Duration::from_secs(dt.timestamp() as u64))
        })
}

fn print_usage() {
    safe_println(format_args!("Usage: {} -[OPTIONS] [FILENAME..]", package_name!()));
    safe_println(format_args!("     a                   Change access time only"));
    safe_println(format_args!("     m                   Change modification time only"));
    safe_println(format_args!("     c, --no-create      Do not create file if not exist"));
    safe_println(format_args!("     r FILE              Use timestamp from FILE"));
    safe_println(format_args!("     t STAMP             [[CC]YY]MMDDhhmm[.ss]"));
    safe_println(format_args!("     d STRING            Specify time format (example: 2023-06-15 20:00)"));
    safe_println(format_args!("     --help              Show help"));
    safe_println(format_args!("     --version           Show version"));
}