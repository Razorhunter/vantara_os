use std::fs::File;
use std::io::{BufRead, BufReader};
use std::fs::OpenOptions;
use std::io::Write;

pub struct GroupEntry {
    pub groupname: String,
    pub gid: u32,
}

const DEFAULT_GROUP_FILE: &str = "/etc/group";

pub fn get_group_entry(username: &str) -> Option<GroupEntry> {
    let file = File::open(DEFAULT_GROUP_FILE).ok()?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        if let Ok(entry) = line {
            let fields: Vec<&str> = entry.split(':').collect();

            if fields.len() >= 7 && fields[0] == username {
                return Some(GroupEntry {
                    groupname: fields[0].to_string(),
                    gid: fields[2].parse().ok()?,
                });
            }
        }
    }
    None
}

pub fn add_user_to_group_file(username: &str, gid: u32) -> std::io::Result<()> {
    let entry = format!(
        "{}:x:{}:\n",
        username, gid
    );

    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(DEFAULT_GROUP_FILE)?;

    writeln!(file, "{}", entry)?;

    Ok(())
}