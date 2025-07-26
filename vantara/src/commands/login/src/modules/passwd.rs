use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct PasswdEntry {
    pub username: String,
    pub uid: u32,
    pub gid: u32,
    pub fullname: String,
    pub home: String,
    pub shell: String,
}

const DEFAULT_PASSWD_FILE: &str = "/etc/passwd";

pub fn get_passwd_entry(username: &str) -> Option<PasswdEntry> {
    let file = File::open(DEFAULT_PASSWD_FILE).ok()?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        if let Ok(entry) = line {
            let fields: Vec<&str> = entry.split(':').collect();
            if fields.len() >= 7 && fields[0] == username {
                return Some(PasswdEntry {
                    username: fields[0].to_string(),
                    uid: fields[2].parse().ok()?,
                    gid: fields[3].parse().ok()?,
                    fullname: fields[4].to_string(),
                    home: fields[5].to_string(),
                    shell: fields[6].to_string(),
                });
            }
        }
    }
    None
}