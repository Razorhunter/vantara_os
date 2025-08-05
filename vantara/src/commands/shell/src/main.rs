mod kill;

use dirs::home_dir;
use rustyline::{Editor, Helper, CompletionType, Config, Context};
use rustyline::completion::{Completer, Pair};
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::{Validator, ValidationContext, ValidationResult};
use std::env;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use vantara::{safe_println, safe_eprintln};
use std::fmt::Write as _;
use std::process::{exit};
use std::os::unix::net::UnixStream;
use std::io::Read;

const DEFAULT_SOCKET_PATH: &str = "/run/systemd.sock";

struct DirCompleter;

impl Completer for DirCompleter {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), rustyline::error::ReadlineError> {
        let (start, path_prefix) = match line[..pos].rfind(' ') {
            Some(idx) => (idx + 1, &line[idx + 1..pos]),
            None => (0, &line[..pos]),
        };

        let prefix = Path::new(path_prefix);
        let base = if prefix.is_absolute() {
            PathBuf::from(prefix)
        } else {
            env::current_dir().unwrap().join(prefix)
        };

        let parent = base.parent().unwrap_or_else(|| Path::new("."));

        let completions = match fs::read_dir(parent) {
            Ok(read_dir) => read_dir,
            Err(_) => match fs::read_dir(".") {
                Ok(fallback) => fallback,
                Err(_) => return Ok((start, Vec::new())), // safely fail
            }
        }
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                let file_name = e.file_name().into_string().ok()?;
                let full_path = e.path();
                if full_path.is_dir() && file_name.starts_with(prefix.file_name()?.to_str()?) {
                    Some(Pair {
                        display: file_name.clone() + "/",
                        replacement: file_name + "/"
                    })
                } else {
                    None
                }
            })
        })
        .collect();
        Ok((start, completions))
    }
}

impl Helper for DirCompleter {}

impl Validator for DirCompleter {
    fn validate(&self, _: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        Ok(ValidationResult::Valid(None))
    }
}

impl Highlighter for DirCompleter {}

impl Hinter for DirCompleter {
    type Hint = String;
}

fn main() {
    let username = std::env::var("USER").unwrap_or("user".to_string());
    let home_dir = std::env::var("HOME").unwrap_or("/".to_string());
    let profile_path = format!("{}/.profile", home_dir);
    let _aliases = load_profile(&profile_path);

    //Defaulting binary PATH
    env::set_var("PATH", "/bin:/usr/bin:/sbin:/usr/sbin");

    let config = Config::builder()
        .completion_type(CompletionType::List)
        .build();
    let mut rl = match Editor::with_config(config) {
        Ok(editor) => editor,
        Err(e) => {
            safe_eprintln(format_args!("Editor init failed: {}", e));
            return;
        }
    };
    rl.set_helper(Some(DirCompleter));

    loop {
        let curr_dir = env::current_dir().unwrap();
        let mut prompt = String::with_capacity(128);
        let _ = write!(
            &mut prompt,
            "\x1b[38;2;0;255;0m{}@host\x1b[0m:\x1b[38;2;0;119;255m{}\x1b[0m$ ",
            username,
            get_display_path(&curr_dir)
        );

        match rl.readline(&prompt) {
            Ok(line) => {

                let input = line.trim();
                if input.is_empty() {
                    continue;
                }

                let _ = rl.add_history_entry(input);

                let mut parts = input.split_whitespace();
                let command = parts.next().unwrap_or("");

                match command {
                    "cd" => {
                        let new_dir = parts.next().unwrap_or(&home_dir);
                        if let Err(e) = env::set_current_dir(new_dir) {
                            safe_eprintln(format_args!("cd: {}: {}", new_dir, e));
                        }
                    },
                    "env" => {
                        for (key, value) in env::vars() {
                            safe_println(format_args!("{}={}", key, value));
                        }
                    },
                    "clear" => {
                        print!("\x1B[2J\x1B[1;1H");
                        std::io::stdout().flush().unwrap();
                    },
                    "exit" => {
                        safe_println(format_args!("Goodbye, {}!", username));
                        exit(0);
                    },
                    "history" => {
                        for (i, cmd) in rl.history().iter().enumerate() {
                            safe_println(format_args!("{:>4} {}", i + 1, cmd));
                        }
                    },
                    "service" => {
                        let first = parts.next();
                        let second = parts.next();

                        match (first, second) {
                            (Some("list"), None) => {
                                run_service_command("list", None);
                            },
                            (Some(name), Some("enable")) => {
                                run_service_command("enable", Some(name));
                            },
                            (Some(name), Some("disable")) => {
                                run_service_command("disable", Some(name));
                            },
                            (Some(name), Some("status")) => {
                                run_service_command("status", Some(name));
                            },
                            (Some(name), Some("start")) => {
                                run_service_command("start", Some(name));
                            },
                            (Some(name), Some("stop")) => {
                                run_service_command("stop", Some(name));
                            },
                            (Some(name), Some("restart")) => {
                                run_service_command("restart", Some(name));
                            },
                            _ => {
                                safe_eprintln(format_args!(
                                    "Usage:\n  service list\n  service <name> <status|start|stop|restart|enable|disable>"
                                ));
                            }
                        }
                    },
                    "kill" => {
                        let first = parts.next();
                        let second = parts.next();

                        match (first, second) {
                            (Some(signal_or_pid), Some(pid_str)) => {
                                // Dua argumen: anggap pertama = signal, kedua = PID
                                crate::kill::kill_process(signal_or_pid, pid_str);
                            }
                            (Some(pid_str), None) => {
                                // Satu argumen sahaja: anggap default signal (SIGTERM)
                                crate::kill::kill_process("SIGTERM", pid_str);
                            }
                            _ => {
                                safe_eprintln(format_args!("Usage: kill [-SIGNAL] <PID>"));
                            }
                        }
                    },
                    _ => {
                        let parts: Vec<&str> = input.split_whitespace().collect();
                        if parts.is_empty() {
                            continue;
                        }
                        run_pipeline_command(input);
                    }
                }
            }
            Err(e) => {
                safe_println(format_args!("sh: error: {}", e));
                exit(1);
            }
        }
    }
}

fn run_pipeline_command(input: &str) {
    let commands: Vec<&str> = input.trim().split('|').map(str::trim).collect();

    if commands.is_empty() {
        return;
    }

    let mut previous_stdout = None;
    let mut children = Vec::new();

    for (i, cmd_str) in commands.iter().enumerate() {
        let mut parts = cmd_str.split_whitespace();
        let cmd = match parts.next() {
            Some(c) => c,
            None => continue,
        };

        let args: Vec<&str> = parts.collect();

        let mut command = Command::new(cmd);
        command.args(args);

        if let Some(stdout) = previous_stdout.take() {
            command.stdin(Stdio::from(stdout));
        }

        if i == commands.len() - 1 {
            command.stdout(Stdio::inherit());
        } else {
            command.stdout(Stdio::piped());
        }

        match command.spawn() {
            Ok(mut child) => {
                // Ambil stdout dulu sebelum push child
                let stdout = if i != commands.len() - 1 {
                    child.stdout.take()
                } else {
                    None
                };

                previous_stdout = stdout;
                children.push(child);
            }
            Err(_) => {
                safe_eprintln(format_args!("Command '{}' not found", cmd));
                return;
            }
        }

    }

    for mut child in children {
        let _ = child.wait();
    }
}

fn run_service_command(cmd: &str, name: Option<&str>) {
    let full_command = match name {
        Some(n) => format!("{} {}", cmd, n),
        None => cmd.to_string(),
    };

    match UnixStream::connect(DEFAULT_SOCKET_PATH) {
        Ok(mut stream) => {
            if let Err(e) = stream.write_all(full_command.as_bytes()) {
                safe_eprintln(format_args!("Failed to write to socket: {}", e));
                return;
            }

            let mut buffer = [0u8; 1024];
            let mut response = String::new();
            loop {
                match stream.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(n) => response.push_str(&String::from_utf8_lossy(&buffer[..n])),
                    Err(e) => {
                        safe_eprintln(format_args!("Failed to read from socket: {}", e));
                        break;
                    }
                }
            }

            println!("{}", response);
        }
        Err(e) => {
            safe_eprintln(format_args!("Failed to connect to minisystemd: {}", e));
        }
    }
}

fn get_display_path(path: &Path) -> String {
    if let Some(home) = home_dir() {
        if let Ok(stripped) = path.strip_prefix(&home) {
            return format!("~/{}", stripped.display());
        }
    }
    path.display().to_string()
}

fn load_profile(path: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    if let Ok(file) = File::open(path) {
        let reader = BufReader::new(file);
        for line in reader.lines().flatten() {
            if let Some((k, v)) = line.strip_prefix("alias ")
            .and_then(|s| s.split_once('='))
            .map(|(k, v)| (k.trim().to_string(), v.trim_matches('"').to_string()))
            {
                map.insert(k, v);
            }
        }
    }
    map
}
