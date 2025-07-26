use std::collections::HashMap;

pub struct AuthContext {
    pub username: String,
    pub password: String,
    pub metadata: HashMap<String, String>, // untuk info tambahan
}

#[derive(Debug)]
pub enum AuthResult {
    Success,
    Failure(String), // alasan kegagalan
}

pub trait AuthModule {
    /// Fasa auth: semak password, fingerprint, 2FA, etc.
    fn auth(&self, ctx: &mut AuthContext) -> AuthResult;
    /// Fasa account: check expiry, status akaun, dsb.
    fn account(&self, ctx: &mut AuthContext) -> AuthResult;
    /// Fasa session: setup environment, log session, dsb.
    fn session(&self, ctx: &mut AuthContext) -> AuthResult;
}

/// Fungsi utama untuk jalankan login flow:
pub fn run_login(modules: &[Box<dyn AuthModule>], ctx: &mut AuthContext) -> bool {
    // Fasa authentication
    for module in modules {
        match module.auth(ctx) {
            AuthResult::Success => continue,
            AuthResult::Failure(reason) => {
                println!("Auth failed: {}", reason);
                return false;
            }
        }
    }

    // Fasa account checking
    for module in modules {
        match module.account(ctx) {
            AuthResult::Success => continue,
            AuthResult::Failure(reason) => {
                println!("Account check failed: {}", reason);
                return false;
            }
        }
    }

    // Fasa session setup
    for module in modules {
        match module.session(ctx) {
            AuthResult::Success => continue,
            AuthResult::Failure(reason) => {
                println!("Session setup failed: {}", reason);
                return false;
            }
        }
    }

    true
}
