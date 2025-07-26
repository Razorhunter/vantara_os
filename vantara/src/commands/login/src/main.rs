mod auth;
mod modules;

use modules::session_log::log_logout;
use auth::{run_login, AuthContext};
use modules::{auth_unix::AuthUnix, session_logger::SessionLogger, account_expiry::AccountExpiry};
use crate::modules::passwd::get_passwd_entry;
use std::io::{self, stdin, Write};
use std::process::Command;
use vantara::{safe_print, safe_println, safe_eprintln, read_password};

fn main() {
    let modules: Vec<Box<dyn auth::AuthModule>> = vec![
        Box::new(AuthUnix::new()),
        Box::new(SessionLogger::new()),
        Box::new(AccountExpiry::new()),
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
                let _ = Command::new(&user.shell)
                    .spawn()
                    .unwrap()
                    .wait();
            } else {
                safe_eprintln(format_args!("User entry not found"));
            }
        } else {
            safe_println(format_args!("Please try again"));
        }
        log_logout(&ctx.username);
    }
}
