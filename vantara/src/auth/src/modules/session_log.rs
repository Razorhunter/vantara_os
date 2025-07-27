use chrono::Local;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::fs::MetadataExt;
use libc::{isatty, ttyname};
use std::ffi::CStr;
use std::fs::File;
use std::io::{BufReader, BufRead};
use vantara::{safe_println};

const DEFAULT_USER_LOG_PATH: &str = "/var/.session.log";

#[derive(Debug, Clone)]
struct SessionEntry {
    username: String,
    tty: String,
    ip: String,
    time: String,
}

fn get_tty() -> String {
    unsafe {
        if isatty(0) == 1 {
            let ptr = ttyname(0);
            if !ptr.is_null() {
                let cstr = CStr::from_ptr(ptr);
                return cstr.to_string_lossy().into_owned();
            }
        }
    }
    "unknown".to_string()
}

fn get_uid() -> u32 {
    std::fs::metadata("/proc/self").map(|m| m.uid()).unwrap_or(0)
}

pub fn log_login(username: &str) {
    write_log("LOGIN", username);
}

pub fn log_logout(username: &str) {
    write_log("LOGOUT", username);
}

fn write_log(event: &str, username: &str) {
    let now = Local::now();
    let tty = get_tty();
    let uid = get_uid();
    let ip = get_env_ip().unwrap_or_else(|| "local".to_string());

    let log_line = format!(
        "[{}] user={} uid={} tty={} ip={} time={}\n",
        event,
        username,
        uid,
        tty,
        ip,
        now.format("%Y-%m-%d %H:%M:%S")
    );

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(DEFAULT_USER_LOG_PATH)
        .unwrap_or_else(|_| panic!("Failed to open session log file"));

    let _ = file.write_all(log_line.as_bytes());
}

fn get_env_ip() -> Option<String> {
    std::env::var("SSH_CLIENT")
        .ok()
        .and_then(|v| v.split_whitespace().next().map(|s| s.to_string()))
}

pub fn print_logged_in_users() {
    let file = File::open(DEFAULT_USER_LOG_PATH)
        .unwrap_or_else(|_| panic!("Failed to open session log file"));

    let reader = BufReader::new(file);
    let mut active_sessions: Vec<SessionEntry> = vec![];

    for line in reader.lines().flatten() {
        if line.starts_with("[LOGIN]") {
            if let Some(entry) = parse_log_line(&line) {
                active_sessions.push(entry);
            }
        } else if line.starts_with("[LOGOUT]") {
            if let Some(entry) = parse_log_line(&line) {
                // Cari sesi padan (username + tty + ip)
                if let Some(pos) = active_sessions.iter().position(|s| {
                    s.username == entry.username && s.tty == entry.tty && s.ip == entry.ip
                }) {
                    active_sessions.remove(pos);
                }
            }
        }
    }

    for session in active_sessions {
        safe_println(format_args!(
            "{:<12} {:<15} {:<20} {}",
            session.username, session.tty, session.ip, session.time
        ));
    }
}

fn parse_log_line(line: &str) -> Option<SessionEntry> {
    let mut username = String::from("unknown");
    let mut tty = String::from("unknown");
    let mut ip = String::from("unknown");
    let mut time = String::from("unknown");

    let parts: Vec<&str> = line.split_whitespace().collect();
    let mut i = 0;

    while i < parts.len() {
        let part = parts[i];

        if let Some(val) = part.strip_prefix("user=") {
            username = val.to_string();
        } else if let Some(val) = part.strip_prefix("tty=") {
            tty = val.to_string();
        } else if let Some(val) = part.strip_prefix("ip=") {
            ip = val.to_string();
        } else if part.starts_with("time=") {
            let date_part = part.strip_prefix("time=").unwrap_or("unknown");
            let time_part = if i + 1 < parts.len() { parts[i + 1] } else { "unknown" };
            time = format!("{} {}", date_part, time_part);
            i += 1; // skip time_part
        }

        i += 1;
    }

    Some(SessionEntry {
        username,
        tty,
        ip,
        time,
    })
}
