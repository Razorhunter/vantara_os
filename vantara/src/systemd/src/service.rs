use std::fs;
use std::path::Path;
use std::ffi::CString;
use nix::unistd::{fork, ForkResult, execv, Pid};
use crate::{safe_println, safe_eprintln};

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
        if !self.enabled {
            safe_println(format_args!("[INIT] Skipping disabled service {}", self.name));
            return;
        }

        match unsafe { fork() } {
            Ok(ForkResult::Parent { child }) => {
                safe_println(format_args!("[INIT] Started service {} with PID {}", self.name, child));
                self.pid = Some(child);
            }
            Ok(ForkResult::Child) => {
                let exec_path = CString::new(self.exec.clone()).unwrap();
                let args = [exec_path.clone()];
                execv(&exec_path, &args).expect(&format!("[INIT] Failed to exec {:?}", args));
            }
            Err(err) => {
                safe_eprintln(format_args!("[INIT] Failed to create service for {}: {}", self.name, err));
            }
        }
    }
}
