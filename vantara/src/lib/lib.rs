use chrono_tz::Tz;
use glob::glob;
use std::io::{self, Write, Result};
use std::fs::{read_to_string, write};

#[macro_export]
macro_rules! package_name {
    () => {
        env!("CARGO_PKG_NAME");
    };
}


#[macro_export]
macro_rules! print_version {
    () => {
        println!("   Package name: {}", env!("CARGO_PKG_NAME"));
        println!("    Description: {}", env!("CARGO_PKG_DESCRIPTION"));
        println!("        Version: v{}", env!("CARGO_PKG_VERSION"));
        println!("        License: {}", env!("CARGO_PKG_LICENSE"));
        println!("        Authors: {}", env!("CARGO_PKG_AUTHORS"));
    };
}

pub fn show_boot_banner() {
    safe_println(format_args!("{}", r#"
    __     __          _                  
    \ \   / /_ _ _ __ | |_ __ _ _ __ __ _ 
     \ \ / / _` | '_ \| __/ _` | '__/ _` |
      \ V / (_| | | | | || (_| | | | (_| |
       \_/ \__,_|_| |_|\__\__,_|_|  \__,_|
               Operating System           
    "#));
    safe_println(format_args!("          Welcome to the VanOS {}        \n", env!("CARGO_PKG_VERSION")));
}

pub fn safe_println(args: std::fmt::Arguments) {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let _ = writeln!(handle, "{}", args);
}

pub fn safe_print(args: std::fmt::Arguments) {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let _ = write!(handle, "{}", args);
}

pub fn safe_eprintln(args: std::fmt::Arguments) {
    let stderr = io::stderr();
    let mut handle = stderr.lock();
    let _ = writeln!(handle, "{}", args);
}

pub fn read_file(path: &str) -> Result<String> {
    read_to_string(path)
}

pub fn write_file(path: &str, content: &str) -> Result<()> {
    write(path, content)
}

pub fn expand_wildcards(patterns: &[String]) -> Vec<String> {
    let mut expanded = Vec::new();

    for pattern in patterns {
        if pattern.contains('*') || pattern.contains('?') {
            match glob(pattern) {
                Ok(paths) => {
                    for entry in paths.flatten() {
                        if let Some(p) = entry.to_str() {
                            expanded.push(p.to_string());
                        }
                    }
                }
                Err(_) => {
                    expanded.push(pattern.to_string());
                }
            }
        } else {
            expanded.push(pattern.to_string());
        }
    }

    expanded
}

pub fn confirm(prompt: &str) -> bool {
    safe_print(format_args!("{} (y/N) ", prompt.trim()));
    io::stdout().flush().unwrap();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_ok() {
        matches!(input.trim(), "y" | "Y")
    } else {
        false
    }
}

pub fn get_system_timezone() -> Tz {
    if let Ok(path) = std::fs::read_link("/etc/localtime") {
        if let Ok(rel_path) = path.strip_prefix("/usr/share/zoneinfo/") {
            if let Some(tz_str) = rel_path.to_str() {
                return tz_str.parse().unwrap_or(chrono_tz::UTC);
            }
        }
    }
    chrono_tz::UTC
}
