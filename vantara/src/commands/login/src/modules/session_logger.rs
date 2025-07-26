use crate::auth::{AuthModule, AuthContext, AuthResult};

pub struct SessionLogger {}

impl SessionLogger {
    pub fn new() -> Self {
        SessionLogger {}
    }
}

impl AuthModule for SessionLogger {
    fn auth(&self, _ctx: &mut AuthContext) -> AuthResult {
        // Modul ini tak buat apa dalam fasa auth.
        AuthResult::Success
    }

    fn account(&self, _ctx: &mut AuthContext) -> AuthResult {
        // Modul ini tak buat apa dalam fasa account.
        AuthResult::Success
    }

    fn session(&self, ctx: &mut AuthContext) -> AuthResult {
        // Log session, contohnya ke file atau stdout.
        println!("Logging session for user: {}", ctx.username);
        AuthResult::Success
    }
}
