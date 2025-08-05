mod rtc;

use chrono::{Utc, Local, Datelike, NaiveDate, NaiveTime, NaiveDateTime, DateTime};
use std::env;
use libc::{timeval, settimeofday};
use std::ptr;
use vantara::{safe_println, safe_eprintln, package_name, print_version, get_system_timezone};
use std::process::exit;

#[derive(Default, Debug)]
struct Options {
    utc: bool,
    format: Option<String>,
    set_time: Option<String>,
}

fn main() {
    let mut args = env::args().skip(1).peekable();
    let mut options = Options {
        utc: false,
        format: None,
        set_time: None,
    };

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" => { print_usage(); exit(0); },
            "--version" => { print_version!(); exit(0); },
            "-u" | "--utc" => options.utc = true,
            "-s" | "--set" => {
                if let Some(set_time) = args.next() {
                    options.set_time = Some(set_time);
                } else {
                    safe_eprintln(format_args!("{}: options -s requires an argument", package_name!()));
                    exit(1);
                }
            },
            _ if arg.starts_with('+') => {
                options.format = Some(arg.to_string());
            }
            _ => {
                safe_eprintln(format_args!("{}: unknown option '{}'", package_name!(), arg));
                exit(1);
            }
        }
    }

    if let Some(time_str) = options.set_time {
        if let Some(date) = parse_datetime(&time_str) {
            match set_system_time(date) {
                Ok(_) => safe_println(format_args!("{}: datetime set to '{}'", package_name!(), date)),
                Err(e) => safe_eprintln(format_args!("{}: error set datetime: {}", package_name!(), e))
            }
        }
    }

    if let Some(format) = options.format {
        print_formatted_time(&format, options.utc);
    } else {
        let now = if options.utc {
            Utc::now().with_timezone(&chrono_tz::UTC)
        } else {
            let tz = get_system_timezone(); // jenis chrono_tz::Tz
            Utc::now().with_timezone(&tz)
        };


        safe_println(format_args!("{}", now.format("%a %b %e %Y %T %Z")));
    }
}

fn parse_datetime(input: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S").ok()
}

fn set_system_time(datetime: NaiveDateTime) -> Result<(), String> {
    let timestamp = datetime.timestamp();
    let tv = timeval {
        tv_sec: timestamp,
        tv_usec: 0,
    };

    let result = unsafe { settimeofday(&tv, ptr::null()) };
    if result != 0 {
        return Err(format!("{}: unable to set time. Root require. errno: {}", package_name!(), result));
    }

    Ok(())
}

fn print_formatted_time(format_str: &str, utc: bool) {
    let now = if utc { Utc::now().naive_utc() } else { Local::now().naive_local() };
    let output = now.format(format_str).to_string();
    safe_println(format_args!("{}", output));
}

fn parse_legacy_date(input: &str) -> Option<NaiveDateTime> {
    let (main, second_str) = if let Some(dot_pos) = input.find('.') {
        (&input[..dot_pos], &input[dot_pos + 1..])
    } else {
        (input, "00")
    };

    let second: u32 = second_str.parse().unwrap_or(0);

    let len = main.len();
    if len < 8 {
        return None;
    }

    let month: u32 = main[0..2].parse().ok()?;
    let day: u32 = main[2..4].parse().ok()?;
    let hour: u32 = main[4..6].parse().ok()?;
    let minute: u32 = main[6..8].parse().ok()?;

    let year: i32 = match len {
        8 => Local::now().year(), // default to current year
        10 => {
            let yy: i32 = main[8..10].parse().ok()?;
            let century = Local::now().year() / 100 * 100;
            century + yy
        }
        12 => {
            let cc: i32 = main[8..10].parse().ok()?;
            let yy: i32 = main[10..12].parse().ok()?;
            cc * 100 + yy
        }
        _ => return None,
    };

    let date = NaiveDate::from_ymd_opt(year, month, day)?;
    let time = NaiveTime::from_hms_opt(hour, minute, second)?;
    Some(NaiveDateTime::new(date, time))
}

fn print_usage() {
    safe_println(format_args!("Usage: {} [OPTIONS]", package_name!()));
}
