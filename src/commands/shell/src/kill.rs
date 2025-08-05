use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;
use vantara::{safe_eprintln, safe_println};

pub fn kill_process(signal_str: &str, pid_str: &str) {
    // Parse signal
    let signal = match parse_signal(signal_str) {
        Some(sig) => sig,
        None => {
            safe_eprintln(format_args!("Invalid signal: {}", signal_str));
            return;
        }
    };

    // Parse PID
    let pid: i32 = match pid_str.parse() {
        Ok(num) => num,
        Err(_) => {
            safe_eprintln(format_args!("Invalid PID: {}", pid_str));
            return;
        }
    };

    let pid = Pid::from_raw(pid);
    match kill(pid, signal) {
        Ok(_) => safe_println(format_args!("Sent {:?} to PID {}", signal, pid)),
        Err(e) => safe_eprintln(format_args!("Failed to send signal: {}", e)),
    }
}

fn parse_signal(sig: &str) -> Option<Signal> {
    match sig {
        "-9" | "SIGKILL" => Some(Signal::SIGKILL),
        "-15" | "SIGTERM" => Some(Signal::SIGTERM),
        "-2" | "SIGINT" => Some(Signal::SIGINT),
        _ => None,
    }
}
