use std::collections::HashMap;
use std::env;
use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use vantara::{get_system_timezone};
use chrono::{DateTime, Local, TimeZone, Utc};

fn main() -> io::Result<()> {
    let uid_to_user = read_passwd();
    let uptime = get_uptime();
    let boot_time = get_boot_time();
    let memtotal_kb = get_memtotal_kb();
    let hertz = ticks_per_second();

    println!(
        "{:<8} {:>5} {:>4} {:>4} {:>8} {:>5} {:<8} {:<5} {:<6} {:<8} {}",
        "USER", "PID", "%CPU", "%MEM", "VSZ", "RSS", "TTY", "STAT", "START", "TIME", "COMMAND"
    );

    for entry in fs::read_dir("/proc")? {
        let entry = entry?;
        let pid_str = entry.file_name().to_string_lossy().to_string();
        let pid: u32 = match pid_str.parse() {
            Ok(p) => p,
            Err(_) => continue,
        };

        let stat_path = format!("/proc/{}/stat", pid);
        let statm_path = format!("/proc/{}/statm", pid);
        let status_path = format!("/proc/{}/status", pid);

        let stat = fs::read_to_string(&stat_path).ok();
        let statm = fs::read_to_string(&statm_path).ok();
        let status = fs::read_to_string(&status_path).ok();

        if stat.is_none() || statm.is_none() || status.is_none() {
            continue;
        }

        let stat = stat.unwrap();
        let statm = statm.unwrap();
        let status = status.unwrap();

        let parts: Vec<&str> = stat.split_whitespace().collect();
        if parts.len() < 22 {
            continue;
        }

        let utime: u64 = parts[13].parse().unwrap_or(0);
        let stime: u64 = parts[14].parse().unwrap_or(0);
        let starttime: u64 = parts[21].parse().unwrap_or(0);
        let vsz: u64 = parts[22].parse().unwrap_or(0);
        let stat_state = parts[2];

        let rss_kb: u64 = statm.split_whitespace().nth(1).and_then(|v| v.parse().ok()).unwrap_or(0) * 4;

        let mut uid: u32 = 0;
        for line in status.lines() {
            if line.starts_with("Uid:") {
                uid = line.split_whitespace().nth(1).unwrap_or("0").parse().unwrap_or(0);
                break;
            }
        }
        let user = uid_to_user.get(&uid).cloned().unwrap_or(uid.to_string());

        let cpu_time = utime + stime;
        let seconds = uptime - (starttime as f64 / hertz as f64);
        let pcpu = if seconds > 0.0 {
            (cpu_time as f64 / hertz as f64) / seconds * 100.0
        } else {
            0.0
        };

        let pmem = (rss_kb as f64 / memtotal_kb as f64) * 100.0;

        let tty = get_tty(pid);
        let start_fmt = format_start_time(starttime, boot_time);
        let time_fmt = format_time(cpu_time);
        let cmd = get_cmdline(pid);

        println!(
            "{:<8} {:>5} {:>4.1} {:>4.1} {:>8} {:>5} {:<8} {:<5} {:<6} {:<8} {}",
            user,
            pid,
            pcpu,
            pmem,
            vsz,
            rss_kb,
            tty,
            stat_state,
            start_fmt,
            time_fmt,
            cmd
        );
    }

    Ok(())
}

fn read_passwd() -> HashMap<u32, String> {
    let mut map = HashMap::new();
    if let Ok(content) = fs::read_to_string("/etc/passwd") {
        for line in content.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() > 2 {
                if let Ok(uid) = parts[2].parse::<u32>() {
                    map.insert(uid, parts[0].to_string());
                }
            }
        }
    }
    map
}

fn get_uptime() -> f64 {
    fs::read_to_string("/proc/uptime")
        .ok()
        .and_then(|s| s.split_whitespace().next()?.parse::<f64>().ok())
        .unwrap_or(0.0)
}

fn get_memtotal_kb() -> u64 {
    fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|line| line.starts_with("MemTotal"))
                .and_then(|line| line.split_whitespace().nth(1))
                .and_then(|v| v.parse::<u64>().ok())
        })
        .unwrap_or(1)
}

fn ticks_per_second() -> u64 {
    unsafe { libc::sysconf(libc::_SC_CLK_TCK) as u64 }
}

fn format_time(total_ticks: u64) -> String {
    let seconds = total_ticks / ticks_per_second();
    let h = seconds / 3600;
    let m = (seconds % 3600) / 60;
    let s = seconds % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}

fn get_tty(pid: u32) -> String {
    let fd_path = format!("/proc/{}/fd/0", pid);
    if let Ok(target) = fs::read_link(fd_path) {
        if let Some(path) = target.to_str() {
            if path.contains("/pts/") || path.contains("/tty") {
                return path.to_string();
            }
        }
    }
    "?".to_string()
}

fn get_boot_time() -> u64 {
    fs::read_to_string("/proc/stat")
        .ok()
        .and_then(|s| {
            for line in s.lines() {
                if line.starts_with("btime") {
                    return line.split_whitespace().nth(1)?.parse::<u64>().ok();
                }
            }
            None
        })
        .unwrap_or(0)
}

fn format_start_time(start_ticks: u64, boot_time: u64) -> String {
    let seconds = boot_time + (start_ticks / ticks_per_second());
    let booted = UNIX_EPOCH + Duration::from_secs(seconds);

    // Tukar ke zon masa tempatan
    let local_time: DateTime<Local> = DateTime::<Utc>::from(booted).with_timezone(&Local);

    // Kira beza masa dengan sekarang
    let now = Local::now();
    let since = now.timestamp() - local_time.timestamp();

    if since < 86400 {
        local_time.format("%H:%M").to_string()
    } else {
        local_time.format("%b%d").to_string()
    }
}

fn get_cmdline(pid: u32) -> String {
    let path = format!("/proc/{}/cmdline", pid);
    fs::read(path)
        .ok()
        .and_then(|v| {
            let s = v
                .split(|&b| b == 0)
                .filter(|s| !s.is_empty())
                .map(|s| String::from_utf8_lossy(s).to_string())
                .collect::<Vec<_>>()
                .join(" ");
            if s.is_empty() {
                Some(format!("[{}]", pid))
            } else {
                Some(s)
            }
        })
        .unwrap_or_else(|| "[unknown]".to_string())
}
