use chrono::Local;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::fs::MetadataExt;
use libc::{isatty, ttyname};
use std::ffi::CStr;

const DEFAULT_USER_LOG_PATH: &str = "/var/.session.log";

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
