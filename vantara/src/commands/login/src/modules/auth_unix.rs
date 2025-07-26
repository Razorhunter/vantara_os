use crate::auth::{AuthModule, AuthContext, AuthResult};
use crate::modules::passwd::get_passwd_entry;
use crate::modules::shadow::{get_shadow_entry, hash_password_with_salt};

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
        // Contoh: Check jika username tak kosong.
        if !ctx.username.is_empty() {
            AuthResult::Success
        } else {
            AuthResult::Failure("Empty username".into())
        }
    }

    fn session(&self, _ctx: &mut AuthContext) -> AuthResult {
        // Pada fasa ini, kau mungkin set environment, mounted home, dsb.
        AuthResult::Success
    }
}
