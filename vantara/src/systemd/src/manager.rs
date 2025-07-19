use crate::{safe_println, safe_eprintln};
use crate::service::Service;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::os::unix::net::UnixListener;
use std::io::{Read, Write};
use std::sync::{Mutex, Arc};
use std::thread;
use std::os::unix::fs::PermissionsExt;

pub const DEFAULT_SERVICE_AVAILABLE_PATH: &str = "/etc/service/available";
pub const DEFAULT_SERVICE_ENABLED_PATH: &str = "/etc/service/enabled";
const DEFAULT_SOCKET_PATH: &str = "/run/systemd.sock";

pub struct ServiceManager {
    pub services: HashMap<String, Service>,
}

impl ServiceManager {
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(ServiceManager {
            services: HashMap::new(),
        }))
    }

    pub fn load_services(self_arc: Arc<Mutex<Self>>) {
        // Remove old socket
        if Path::new(DEFAULT_SOCKET_PATH).exists() {
            fs::remove_file(DEFAULT_SOCKET_PATH).expect("Failed to remove existing socket");
        }

        let listener = UnixListener::bind(DEFAULT_SOCKET_PATH).expect("Failed to bind to socket");
        fs::set_permissions(DEFAULT_SOCKET_PATH, fs::Permissions::from_mode(0o660)).unwrap();

        safe_println(format_args!("[INIT] Listening on {}", DEFAULT_SOCKET_PATH));

        // Load services
        {
            let mut manager = self_arc.lock().unwrap();
            let entries = fs::read_dir(DEFAULT_SERVICE_AVAILABLE_PATH)
                .unwrap_or_else(|_| panic!("[INIT] Cannot open {}", DEFAULT_SERVICE_AVAILABLE_PATH));

            for entry in entries {
                let path = match entry {
                    Ok(e) => e.path(),
                    Err(e) => {
                        safe_eprintln(format_args!("[INIT] Error reading entry: {}", e));
                        continue;
                    }
                };

                if path.extension().map_or(false, |e| e == "service") {
                    match Service::from_file(&path) {
                        Ok(service) => {
                            safe_println(format_args!("[INIT] Loading config for service {}", service.name));
                            manager.services.insert(service.name.clone(), service);
                        }
                        Err(err) => {
                            safe_eprintln(format_args!("[INIT] Error loading {}: {}", path.display(), err));
                        }
                    }
                }
            }
        }

        // IPC thread
        let listener_arc = Arc::new(listener);
        let sm_clone = Arc::clone(&self_arc);

        thread::spawn(move || {
            for stream in listener_arc.incoming() {
                match stream {
                    Ok(mut socksocketet) => {
                        let mut buffer = [0u8; 512];
                        if let Ok(n) = socket.read(&mut buffer) {
                            let input = String::from_utf8_lossy(&buffer[..n]).to_string();

                            let response = {
                                let mut manager = sm_clone.lock().unwrap();
                                manager.handle_command(&input)
                            };

                            socket.write_all(response.as_bytes()).ok();
                        }
                    }
                    Err(e) => {
                        safe_eprintln(format_args!("Socket error: {}", e));
                    }
                }
            }
        });
    }

    fn handle_command(&mut self, input: &str) -> String {
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.is_empty() {
            return "Invalid command\n".into();
        }

        match parts[0] {
            "start" => {
                if let Some(name) = parts.get(1) {
                    self.start_service(name);
                    format!("Started {}\n", name)
                } else {
                    "start <service> required\n".into()
                }
            }
            "stop" => {
                if let Some(name) = parts.get(1) {
                    self.stop_service(name);
                    format!("Stopped {}\n", name)
                } else {
                    "stop <service> required\n".into()
                }
            }
            "restart" => {
                if let Some(name) = parts.get(1) {
                    self.stop_service(name);
                    self.start_service(name);
                    format!("Restarted {}\n", name)
                } else {
                    "restart <service> required\n".into()
                }
            }
            "enable" => {
                if let Some(name) = parts.get(1) {
                    self.enable_service(name);
                    format!("Enabled {}\n", name)
                } else {
                    "enable <service> required\n".into()
                }
            }
            "disable" => {
                if let Some(name) = parts.get(1) {
                    self.disable_service(name);
                    format!("Disabled {}\n", name)
                } else {
                    "disable <service> required\n".into()
                }
            }
            "status" => {
                if let Some(name) = parts.get(1) {
                    self.status_service(name);
                    format!("")
                } else {
                    "status <service> required\n".into()
                }
            }
            "list" => self.list_services(),
            _ => "Unknown command\n".into(),
        }
    }

    pub fn start_enabled_services(&mut self) {
        for svc in Self::read_enabled_services(DEFAULT_SERVICE_ENABLED_PATH) {
            safe_println(format_args!("[INIT] Starting service {}", svc.name));
            self.start_service(&svc.name);
        }
    }

    fn read_enabled_services(enabled_dir: &str) -> Vec<Service> {
        let mut services = Vec::new();

        let entries = match fs::read_dir(enabled_dir) {
            Ok(e) => e,
            Err(e) => {
                safe_eprintln(format_args!("Cannot open {}: {}", enabled_dir, e));
                return services;
            }
        };

        for entry in entries {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let path = entry.path();

            // Abaikan kalau bukan .service atau .toml
            if path.extension().map_or(false, |e| e != "service" && e != "toml") {
                continue;
            }

            // Cuba resolve symlink
            match fs::read_link(&path) {
                Ok(target_path) => {
                    // Pastikan path relatif ke enabled_dir
                    let abs_target = if target_path.is_relative() {
                        let mut abs = PathBuf::from(enabled_dir);
                        abs.push(target_path);
                        abs
                    } else {
                        target_path
                    };

                    match Service::from_file(&abs_target) {
                        Ok(mut service) => {
                            service.enabled = true; // override walaupun file tak tulis Enabled=true
                            services.push(service);
                        }
                        Err(e) => {
                            safe_eprintln(format_args!("[INIT] Failed to load service from {} (symlink from {}): {}", abs_target.display(), path.display(), e));
                        }
                    }
                }
                Err(e) => {
                    safe_eprintln(format_args!("[INIT] Failed to resolve symlink {}: {}", path.display(), e));
                }
            }
        }

        services
    }

    fn start_service(&mut self, name: &str) {
        if let Some(service) = self.services.get_mut(name) {
            service.start();
        } else {
            safe_eprintln(format_args!("Service '{}' not found", name));
        }
    }

    fn stop_service(&mut self, name: &str) {
        if let Some(service) = self.services.get_mut(name) {
            service.stop();
        } else {
            safe_eprintln(format_args!("Service '{}' not found", name));
        }
    }

    fn enable_service(&mut self, name: &str) {
        if let Some(service) = self.services.get_mut(name) {
            let source = format!("{}/{}.service", DEFAULT_SERVICE_AVAILABLE_PATH, name);
            let target = format!("{}/{}.service", DEFAULT_SERVICE_ENABLED_PATH, name);

            service.enable(&source, &target);
        } else {
            safe_eprintln(format_args!("Service '{}' not found", name));
        }
    }

    fn disable_service(&mut self, name: &str) {
        if let Some(service) = self.services.get_mut(name) {
            let target = format!("{}/{}.service", DEFAULT_SERVICE_ENABLED_PATH, name);

            service.disable(&target);
        } else {
            safe_eprintln(format_args!("Service '{}' not found", name));
        }
    }

    fn list_services(&self) -> String {
        let mut output = String::new();
        for (name, svc) in &self.services {
            output += &format!(
                "[{}] {} [{}] {} {}\n",
                if svc.pid.is_some() { '*' } else { ' ' },
                name,
                if svc.enabled { "ENABLED" } else { "DISABLED" },
                if svc.pid.is_some() { "running" } else { "stopped" },
                if let Some(pid) = svc.pid {
                    format!("at PID {}", pid) } else { String::new()
                }
            );
        }
        output
    }

    fn status_service(&mut self, name: &str) {
        if let Some(service) = self.services.get_mut(name) {
            service.status();
        } else {
            safe_eprintln(format_args!("Service '{}' not found", name));
        }
    }
}
