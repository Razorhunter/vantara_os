use crate::auth::{AuthModule, AuthContext, AuthResult};
use crate::modules::session_log::log_login;

pub struct SessionLogger {}

impl SessionLogger {
    pub fn new() -> Self {
        SessionLogger {}
    }
}

impl AuthModule for SessionLogger {
    fn auth(&self, _ctx: &mut AuthContext) -> AuthResult {
        AuthResult::Success
    }

    fn account(&self, _ctx: &mut AuthContext) -> AuthResult {
        AuthResult::Success
    }

    fn session(&self, ctx: &mut AuthContext) -> AuthResult {
        log_login(&ctx.username);
        AuthResult::Success
    }
}
