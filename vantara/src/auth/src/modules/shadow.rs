use std::fs::File;
use std::io::{BufRead, BufReader};
use sha2::{Sha512, Digest};

pub struct ShadowEntry {
    pub username: String,
    pub algo_id: String,
    pub salt: String,
    pub hash: String
}

const DEFAULT_SHADOW_FILE: &str = "/etc/shadow";

pub fn get_shadow_entry(username: &str) -> Option<ShadowEntry> {
    let file = File::open(DEFAULT_SHADOW_FILE).ok()?;
    for line in BufReader::new(file).lines().flatten() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 2 && parts[0] == username && parts[1].starts_with('$') {
            let hash_parts: Vec<&str> = parts[1].split('$').collect();
            if hash_parts.len() >= 4 {
                return Some(ShadowEntry {
                    username: hash_parts[0].to_string(),
                    algo_id: hash_parts[1].parse().ok()?,
                    salt: hash_parts[2].to_string(),
                    hash: hash_parts[3].to_string(),
                });
            }
        }
    }
    None
}

pub fn hash_password_with_salt(salt: &str, password: &str) -> String {
    let mut hasher = Sha512::new();
    hasher.update(salt.as_bytes());
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}
