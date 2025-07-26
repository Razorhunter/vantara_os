use crate::auth::{AuthModule, AuthContext, AuthResult};

pub struct AccountExpiry {}

impl AccountExpiry {
    pub fn new() -> Self {
        AccountExpiry {}
    }
}

impl AuthModule for AccountExpiry {
    fn auth(&self, _ctx: &mut AuthContext) -> AuthResult {
        AuthResult::Success
    }

    fn account(&self, ctx: &mut AuthContext) -> AuthResult {
        if ctx.username == "expired_user" {
            AuthResult::Failure("Account expired".into())
        } else {
            AuthResult::Success
        }
    }

    fn session(&self, _ctx: &mut AuthContext) -> AuthResult {
        AuthResult::Success
    }
}
