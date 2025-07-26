mod auth;
mod modules;

use auth::{run_login, AuthContext};
use modules::{auth_unix::AuthUnix, session_logger::SessionLogger};
use crate::modules::passwd::{PasswdEntry, get_passwd_entry};
use std::io::{self, stdin, Write};
use std::process::Command;
use vantara::{safe_print, safe_println, safe_eprintln, read_password };
use libc::{setuid, setgid};

fn main() {
    let modules: Vec<Box<dyn auth::AuthModule>> = vec![
        Box::new(AuthUnix::new()),
        Box::new(SessionLogger::new()),
    ];

    loop {
        safe_print(format_args!("Username: "));
        let _ = io::stdout().flush(); // Ensure the prompt is printed immediately
        let mut username = String::new();
        stdin().read_line(&mut username).unwrap();
        let username = username.trim(); // Remove any trailing newline or spaces

        safe_print(format_args!("Password: "));
        let _ = io::stdout().flush(); // Ensure the prompt is printed immediately
        let password = read_password();

        let mut ctx = AuthContext {
            username: username.to_string(),
            password: password.to_string(),
            metadata: std::collections::HashMap::new(),
        };

        if run_login(&modules, &mut ctx) {
            if let Some(user) = get_passwd_entry(&ctx.username) {
                spawn_shell_as_user(&user);
            } else {
                safe_eprintln(format_args!("User entry not found"));
            }
        } else {
            safe_println(format_args!("Please try again"));
        }
    }
}

fn spawn_shell_as_user(user: &PasswdEntry) {
    unsafe {
        if setgid(user.gid) != 0 {
            safe_eprintln(format_args!("Failed to setgid to {}", user.gid));
        }
        if setuid(user.uid) != 0 {
            safe_eprintln(format_args!("Failed to setuid to {}", user.uid));
        }
    }

    std::env::set_var("HOME", &user.home);
    std::env::set_var("USER", &user.username);
    std::env::set_var("SHELL", &user.shell);

    std::env::set_current_dir(&user.home).unwrap_or_else(|_| {
        safe_eprintln(format_args!("Failed to set home dir to {}", &user.home));
    });

    let _ = Command::new(&user.shell)
        .spawn()
        .unwrap()
        .wait();
}
