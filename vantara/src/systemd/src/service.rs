use std::fs;
use std::path::Path;
use std::ffi::CString;
use nix::unistd::{fork, ForkResult, execv, Pid, setsid};
use crate::{safe_println, safe_eprintln, get_system_timezone};
use std::os::unix::fs::symlink;
use nix::sys::wait::{waitpid, WaitPidFlag};
use std::time::SystemTime;
use chrono::{DateTime, Local};
use crate::manager::{DEFAULT_SERVICE_ENABLED_PATH};

#[derive(Debug)]
#[derive(Clone)]
pub struct Service {
    pub loaded_path: String,
    pub service_type: String,
    pub description: String,
    pub name: String,
    pub exec: String,
    pub enabled: bool,
    pub pid: Option<Pid>,
    pub start_time: Option<SystemTime>,
    pub stop_time: Option<SystemTime>,
}

impl Service {
    pub fn from_file(path: &Path) -> Result<Service, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("[INIT] Failed to read {}: {}", path.display(), e))?;

        let mut name = String::new();
        let mut exec = String::new();
        let mut description = String::new();
        let mut service_type = String::new();
        let loaded_path = format!("{}", path.display());

        for line in content.lines() {
            if line.starts_with("Name=") {
                name = line["Name=".len()..].trim().to_string();
            } else if line.starts_with("ExecStart=") {
                exec = line["ExecStart=".len()..].trim().to_string();
            } else if line.starts_with("Description=") {
                description = line["Description=".len()..].trim().to_string();
            } else if line.starts_with("Type=") {
                service_type = line["Type=".len()..].trim().to_string();
            }
        }

        if name.is_empty() {
            return Err(format!("[INIT] Missing Name in {}", path.display()));
        }
        if exec.is_empty() {
            return Err(format!("[INIT] Missing ExecStart in {}", path.display()));
        }



        let enabled = Path::new(&format!("{}/{}.service", DEFAULT_SERVICE_ENABLED_PATH, name)).exists();

        Ok(Service {
            loaded_path,
            description,
            service_type,
            name,
            exec,
            enabled,
            pid: None,
            start_time: None,
            stop_time: None,
        })
    }

    pub fn start(&mut self) {
        match unsafe { fork() } {
            Ok(ForkResult::Parent { child }) => {
                safe_println(format_args!(
                    "[INIT] Started service {} with PID {}",
                    self.name, child
                ));
                self.pid = Some(child);
                self.start_time = Some(SystemTime::now());
                self.stop_time = None;
            }
            Ok(ForkResult::Child) => {
                let exec_path = CString::new(self.exec.clone()).unwrap();
                let argv0 = CString::new(self.name.clone()).unwrap();
                let args = vec![argv0];

                let _ = setsid();

                execv(&exec_path, &args).unwrap_or_else(|e| {
                    safe_eprintln(format_args!("[INIT] Failed to exec {:?}: {}", self.exec, e));
                    std::process::exit(1);
                });
            }
            Err(err) => {
                safe_eprintln(format_args!(
                    "[INIT] Failed to create service for {}: {}",
                    self.name, err
                ));
            }
        }
    }

    pub fn stop(&mut self) {
        if let Some(pid) = self.pid {
            // Hantar SIGTERM
            let _ = nix::sys::signal::kill(pid, nix::sys::signal::Signal::SIGTERM);
            let _ = waitpid(pid, Some(WaitPidFlag::WNOHANG)); // Cuba "clean up"

            // Tunggu proses mati (loop ringkas)
            for _ in 0..10 {
                std::thread::sleep(std::time::Duration::from_millis(200));
                if nix::sys::signal::kill(pid, None).is_err() {
                    // Proses dah mati
                    self.pid = None;
                    self.stop_time = Some(SystemTime::now());
                    self.start_time = None;
                    safe_println(format_args!("Stopped service {}", self.name));
                    return;
                }
            }

            // Paksa kill
            let _ = nix::sys::signal::kill(pid, nix::sys::signal::Signal::SIGKILL);
            let _ = waitpid(pid, None);
            self.pid = None;
            safe_println(format_args!("Force-stopped service {}", self.name));
        } else {
            safe_println(format_args!("Service '{}' not running", self.name));
        }
    }

    pub fn enable(&mut self, source: &str, target: &str) {
        if Path::new(&target).exists() {
            safe_println(format_args!("Service '{}' already enabled", self.name));
            return;
        }

        match symlink(&source, &target) {
            Ok(_) => { self.enabled = true; safe_println(format_args!("Enabling service '{}'", self.name)) },
            Err(e) => safe_eprintln(format_args!("Failed to enable '{}': {}", self.name, e)),
        }
    }

    pub fn disable(&mut self, target: &str) {
        if !Path::new(&target).exists() {
            safe_println(format_args!("Service '{}' not enabled", self.name));
            return;
        }

        match fs::remove_file(&target) {
            Ok(_) => { self.enabled = false; safe_println(format_args!("Disabling service '{}'", self.name)) },
            Err(e) => safe_eprintln(format_args!("Failed to disable '{}': {}", self.name, e)),
        }
    }

    pub fn status(&mut self) {
        let tz = get_system_timezone();

        safe_println(format_args!("      Loaded at: {}", self.loaded_path));
        safe_println(format_args!("   Service Name: {}", self.name));
        let running = if let Some(pid) = self.pid {
            let is_alive = nix::sys::signal::kill(pid, None).is_ok();
            safe_println(format_args!("            PID: {}", pid));
            is_alive
        } else {
            safe_println(format_args!("            PID: None"));
            false
        };

        safe_println(format_args!("        Enabled: {}", if self.enabled { "Yes" } else { "No" }));
        safe_println(format_args!("        Running: {}", if running { "Yes" } else { "No" }));

        if running {
            if let Some(time) = self.start_time {
                let dt: DateTime<Local> = DateTime::from(time);
                let localtime = dt.with_timezone(&tz);
                safe_println(format_args!("   Active since: {}", localtime.format("%Y-%m-%d %H:%M:%S %Z")));
            }
        } else {
            if let Some(time) = self.stop_time {
                let dt: DateTime<Local> = DateTime::from(time);
                let localtime = dt.with_timezone(&tz);
                safe_println(format_args!(" Inactive since: {}", localtime.format("%Y-%m-%d %H:%M:%S %Z")));
            }
        }
    }
}
