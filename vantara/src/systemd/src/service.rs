use std::fs;
use std::path::Path;
use std::ffi::CString;
use nix::unistd::{fork, ForkResult, execv, Pid};
use crate::{safe_println, safe_eprintln};
use std::os::unix::fs::symlink;
use nix::sys::wait::{waitpid, WaitPidFlag};

#[derive(Debug)]
#[derive(Clone)]
pub struct Service {
    pub name: String,
    pub exec: String,
    pub enabled: bool,
    pub pid: Option<Pid>,
}

impl Service {
    pub fn from_file(path: &Path) -> Result<Service, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("[INIT] Failed to read {}: {}", path.display(), e))?;

        let mut name = String::new();
        let mut exec = String::new();
        let mut enabled = false;

        for line in content.lines() {
            if line.starts_with("Name=") {
                name = line["Name=".len()..].trim().to_string();
            } else if line.starts_with("ExecStart=") {
                exec = line["ExecStart=".len()..].trim().to_string();
            } else if line.starts_with("Enabled=") {
                enabled = line["Enabled=".len()..].trim() == "true";
            }
        }

        if name.is_empty() {
            return Err(format!("[INIT] Missing Name in {}", path.display()));
        }
        if exec.is_empty() {
            return Err(format!("[INIT] Missing ExecStart in {}", path.display()));
        }

        Ok(Service {
            name,
            exec,
            enabled,
            pid: None,
        })
    }

    pub fn start(&mut self) {
        match unsafe { fork() } {
            Ok(ForkResult::Parent { child }) => {
                safe_println(format_args!("[INIT] Started service {} with PID {}", self.name, child));
                self.pid = Some(child);
            }
            Ok(ForkResult::Child) => {
                let exec_path = CString::new(self.exec.clone()).unwrap();
                let args = [exec_path.clone()];
                execv(&exec_path, &args).unwrap_or_else(|e| {
                    safe_eprintln(format_args!("[INIT] Failed to exec {:?}: {}", args, e));
                    std::process::exit(1); // <-- ini penting
                });
            }
            Err(err) => {
                safe_eprintln(format_args!("[INIT] Failed to create service for {}: {}", self.name, err));
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
            Ok(_) => safe_println(format_args!("Enabling service '{}'", self.name)),
            Err(e) => safe_eprintln(format_args!("Failed to enable '{}': {}", self.name, e)),
        }
    }

    pub fn disable(&mut self, target: &str) {
        if !Path::new(&target).exists() {
            safe_println(format_args!("Service '{}' not enabled", self.name));
            return;
        }

        match fs::remove_file(&target) {
            Ok(_) => safe_println(format_args!("Disabling service '{}'", self.name)),
            Err(e) => safe_eprintln(format_args!("Failed to disable '{}': {}", self.name, e)),
        }
    }

    pub fn status(&mut self, is_enabled: bool) {
        safe_println(format_args!("  Service Name: {}", self.name));
        let running = if let Some(pid) = self.pid {
            let is_alive = nix::sys::signal::kill(pid, None).is_ok();
            safe_println(format_args!("           PID: {}", pid));
            is_alive
        } else {
            safe_println(format_args!("           PID: None"));
            false
        };

        safe_println(format_args!("       Enabled: {}", if is_enabled { "Yes" } else { "No" }));
        safe_println(format_args!("       Running: {}", if running { "Yes" } else { "No" }));
    }
}
