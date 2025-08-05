use chrono::{Local, Duration};
use sysinfo::System;
use std::env;
use std::fs;
use std::process::exit;
use vantara::{safe_println, safe_eprintln, package_name, print_version};
use std::collections::HashSet;

const DEFAULT_LOADAVG_PATH: &str = "/proc/loadavg";

struct Options {
    pretty: bool,
    since: bool,
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect(); // skip program name
    let mut options = Options {
        pretty: false,
        since: false,
    };

    for arg in &args {
        match arg.as_str() {
            "--help" => { print_usage(); exit(0); },
            "--version" => { print_version!(); exit(0) },
            _ if arg.starts_with('-') => {
                for c in arg.chars().skip(1) {
                    match c {
                        'p' => options.pretty = true,
                        's' => options.since = true,
                        _ => {
                            safe_eprintln(format_args!("{}: unknow flag -{}", package_name!(), c));
                            exit(1);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let _sys = System::new_all();
    let uptime_secs = System::uptime();

    if options.pretty {
        print_pretty(uptime_secs);
    } else if options.since {
        print_since(uptime_secs);
    } else {
        print_default(uptime_secs);
    }
}

fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;

    if days > 0 {
        format!("{} days, {:02}:{:02}", days, hours, minutes)
    } else {
        format!("{:02}:{:02}", hours, minutes)
    }
}

fn pretty_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;

    let mut parts = Vec::new();
    if days > 0 {
        parts.push(format!("{} day{}", days, if days > 1 { "s" } else { "" }));
    }
    if hours > 0 {
        parts.push(format!("{} hour{}", hours, if hours > 1 { "s" } else { "" }));
    }
    if minutes > 0 {
        parts.push(format!("{} minute{}", minutes, if minutes > 1 { "s" } else { "" }));
    }

    if parts.is_empty() {
        parts.push("less than a minute".to_string());
    }

    format!("up {}", parts.join(", "))
}

fn get_user_count() -> usize {
    let mut users = HashSet::new();

    if let Ok(entries) = fs::read_dir("/proc") {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                    if file_name.chars().all(|c| c.is_digit(10)) {
                        let status_path = path.join("status");
                        if let Ok(content) = fs::read_to_string(status_path) {
                            for line in content.lines() {
                                if line.starts_with("Uid:") {
                                    let uid_str = line.split_whitespace().nth(1).unwrap_or("0");
                                    if let Ok(uid) = uid_str.parse::<u32>() {
                                        users.insert(uid);
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    users.len()
}

fn print_pretty(uptime_secs: u64) {
    safe_println(format_args!("{}", pretty_uptime(uptime_secs)));
}

fn print_since(uptime_secs: u64) {
    let now = Local::now();
    let boot_time = now - Duration::seconds(uptime_secs as i64);
    safe_println(format_args!("System boot time: {}", boot_time.format("%Y-%m-%d %H:%M:%S")));
}

fn print_default(uptime_secs: u64) {
    let now = Local::now().format("%H:%M:%S").to_string();
    let uptime = format_uptime(uptime_secs);
    let user_count = get_user_count();

    let loadavg_str = fs::read_to_string(DEFAULT_LOADAVG_PATH).unwrap();
    let load_parts: Vec<&str> = loadavg_str.split_whitespace().collect();
    let (one, five, fifteen) = (
        load_parts[0].parse::<f32>().unwrap(),
        load_parts[1].parse::<f32>().unwrap(),
        load_parts[2].parse::<f32>().unwrap(),
    );

    safe_println(format_args!(
        "{} up {},  {} users,  load average: {:.2}, {:.2}, {:.2}",
        now,
        uptime,
        user_count,
        one,
        five,
        fifteen
    ));
}

fn print_usage() {
    safe_println(format_args!("Usage: {} -[OPTIONS]", package_name!()));
    safe_println(format_args!("Options:"));
    safe_println(format_args!("     p               Show pretty format"));
    safe_println(format_args!("     s               Show system boot uptime"));
    safe_println(format_args!("     --help          Show this help"));
    safe_println(format_args!("     --version       Show version"));
}
