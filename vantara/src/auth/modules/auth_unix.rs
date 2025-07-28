use crate::auth::{AuthModule, AuthContext, AuthResult};
use crate::auth::modules::passwd::get_passwd_entry;
use crate::auth::modules::shadow::{get_shadow_entry, hash_password_with_salt};
use crate::auth::modules::session_log::get_last_login;
use libc::{setuid, setgid};
use crate::common::{safe_eprintln, safe_println};

pub struct AuthUnix {}

impl AuthUnix {
    pub fn new() -> Self {
        AuthUnix {}
    }
}

impl AuthModule for AuthUnix {
    fn auth(&self, ctx: &mut AuthContext) -> AuthResult {
        match get_passwd_entry(&ctx.username) {
            Some(_) => match get_shadow_entry(&ctx.username) {
                Some(shadow) => {
                    let input_hash = hash_password_with_salt(&shadow.salt, &ctx.password);
                    if input_hash == shadow.hash {
                        if let Some(last) = get_last_login(&ctx.username) {
                            safe_println(format_args!("Last login: {}", last));
                        }
                        AuthResult::Success
                    } else {
                        AuthResult::Failure("Invalid password".into())
                    }
                }
                None => AuthResult::Failure("User not found".into()),
            },
            None => AuthResult::Failure("User not found".into()),
        }
    }

    fn account(&self, ctx: &mut AuthContext) -> AuthResult {
        if !ctx.username.is_empty() {
            AuthResult::Success
        } else {
            AuthResult::Failure("Empty username".into())
        }
    }

    fn session(&self, ctx: &mut AuthContext) -> AuthResult {
        match get_passwd_entry(&ctx.username) {
            Some(user) => {
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

                AuthResult::Success
            },
            None => AuthResult::Failure("User not found".into()),
        }
    }
}
