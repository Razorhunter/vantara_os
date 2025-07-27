use std::os::unix::fs as unix_fs;
use std::fs;
use std::io::{self, Write};
use vantara::{safe_print, safe_eprintln, safe_println};

const DEFAULT_LOCALTIME_PATH: &str = "/etc/localtime";
const DEFAULT_TIMEZONE_PATH: &str = "/etc/timezone";

pub fn set_timezone_interactive() {
    loop {
        safe_print(format_args!("Set timezone (e.g., Asia/Kuala_Lumpur): "));
        io::stdout().flush().unwrap();

        let mut timezone_str = String::new();
        io::stdin().read_line(&mut timezone_str).unwrap();
        let timezone_str = timezone_str.trim(); // buang \n dan space

        let parts: Vec<&str> = timezone_str.splitn(2, '/').collect();
        if parts.len() == 2 {
            let region = parts[0];
            let city = parts[1];
            let zoneinfo_path = format!("/usr/share/zoneinfo/{}/{}", region, city);

            if !fs::metadata(&zoneinfo_path).is_ok() {
                safe_println(format_args!("Time zone not exist: {}", zoneinfo_path));
                continue;
            }

            // Padam symlink lama kalau ada
            let _ = fs::remove_file(DEFAULT_LOCALTIME_PATH);

            // Buat symlink baru
            match unix_fs::symlink(&zoneinfo_path, DEFAULT_LOCALTIME_PATH) {
                Ok(_) => {
                    if let Err(e) = fs::write(DEFAULT_TIMEZONE_PATH, format!("{}\n", timezone_str)) {
                        safe_eprintln(format_args!("Failed to write to '{}': {}", DEFAULT_TIMEZONE_PATH, e));
                    }
                    safe_println(format_args!("Successfully set time zone to '{}'", timezone_str));
                    break; 
                },
                Err(e) => safe_eprintln(format_args!("Failed to set time zone: {}", e)),
            }
        } else {
            safe_println(format_args!("Invalid time zone format: {}", timezone_str));
        }
    }
}