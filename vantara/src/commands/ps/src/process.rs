use users::get_user_by_uid;
use crate::args::Options;
use procfs::CurrentSI;
use rayon::prelude::*;
use procfs::process::Process;
use std::fs;
use std::os::unix::fs::MetadataExt;

#[derive(Debug)]
pub struct ProcInfo {
    pub pid: i32,
    pub user: String,
    pub stat: char,
    pub vsz: u64,
    pub rss: u64,
    pub cmd: String,
    pub cpu_percent: f64,
    pub time: String,
    pub tty: String,
}

pub fn get_processes(args: &Options) -> Vec<ProcInfo> {
    let processes: Vec<Process> = match procfs::process::all_processes() {
        Ok(p) => p.filter_map(|x| x.ok()).collect(),
        Err(_) => return vec![],
    };

    let sys_before = read_system_cpu_total().unwrap_or(0);
    std::thread::sleep(std::time::Duration::from_millis(100));
    let sys_after = read_system_cpu_total().unwrap_or(0);

    processes
        .into_par_iter()
        .filter_map(|p| {
            let stat = p.stat().ok()?;
            let status = p.status().ok()?;
            let uid = status.ruid;

            // Keluarkan proses yang user tak minta
            if !args.show_all {
                // -a: Tunjuk proses dengan TTY
                if args.only_with_tty && stat.tty_nr == 0 {
                    return None;
                }

                // -x: Tunjuk proses TANPA TTY
                if args.only_without_tty && stat.tty_nr != 0 {
                    return None;
                }
            }

            let user = get_user_by_uid(uid)
                .map(|u| u.name().to_string_lossy().into_owned())
                .unwrap_or(uid.to_string());

            let cpu_before = stat.utime + stat.stime;
            let stat_after = p.stat().ok()?;
            let cpu_after = stat_after.utime + stat_after.stime;

            let cpu_percent = if sys_after > sys_before {
                ((cpu_after - cpu_before) as f64 / (sys_after - sys_before) as f64) * 100.0
            } else {
                0.0
            };

            let cmdline = p.cmdline().ok().unwrap_or_default().join(" ");
            let mut cmd = if cmdline.is_empty() {
                stat.comm.clone()
            } else {
                cmdline
            };

            if args.show_all {
                if let Ok(env) = p.environ() {
                    let joined_env = env
                        .into_iter()
                        .filter_map(|(k, v)| {
                            let key = k.to_str()?;
                            let val = v.to_str()?;
                            Some(format!("{}={}", key, val))
                        })
                        .collect::<Vec<String>>()
                        .join(" ");
                    cmd = format!("{} {}", joined_env, cmd);
                }
            }

            let total_time = stat.utime + stat.stime;
            let ticks_per_second = procfs::ticks_per_second();
            let seconds = total_time / ticks_per_second as u64;
            let time = format_duration(seconds);

            let tty = format_tty(stat.tty_nr);

            Some(ProcInfo {
                pid: stat.pid,
                user,
                stat: stat.state,
                vsz: stat.vsize / 1024,
                rss: stat.rss * 4,
                cmd,
                cpu_percent,
                time,
                tty,
            })
        })
        .collect()
}

fn read_system_cpu_total() -> Option<u64> {
    let stat = procfs::KernelStats::current().ok()?;
    let total = stat.total;
    Some(
        total.user
            + total.nice
            + total.system
            + total.idle
            + total.iowait.unwrap_or(0)
            + total.irq.unwrap_or(0)
            + total.softirq.unwrap_or(0)
            + total.steal.unwrap_or(0),
    )
}

fn format_duration(total_secs: u64) -> String {
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

fn format_tty(tty_nr: i32) -> String {
    if tty_nr == 0 {
        return "?".to_string();
    }

    let tty_dev = ((tty_nr >> 8) << 8) | (tty_nr & 0xff); // combine major & minor

    // Cari padanan TTY dalam /dev
    let tty_paths = ["/dev/tty", "/dev/pts", "/dev/console"];

    for path in tty_paths {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten() {
                let meta = match entry.metadata() {
                    Ok(m) => m,
                    Err(_) => continue,
                };
                let rdev = meta.rdev();

                if rdev == tty_dev as u64 {
                    // Return nama TTY macam "pts/1" atau "tty2"
                    if let Some(name) = entry.path().to_str() {
                        return name.replace("/dev/", "");
                    }
                }
            }
        }
    }

    "?".to_string() // fallback
}
