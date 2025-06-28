use std::env;
use std::fs::{self, DirEntry, Metadata};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::time::{UNIX_EPOCH};
use chrono::{DateTime, Local};
use std::process::{exit};
use users::{get_user_by_uid, get_group_by_gid};
use vantara::{package_name, safe_println, safe_eprintln, safe_print, print_version};

struct Options {
    show_hidden: bool,
    long_format: bool,
    recursive: bool,
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut path = if args.len() > 1 { &args[1] } else { "." };
    let mut options = Options {
        show_hidden: false,
        long_format: false,
        recursive: false,
    };

    for arg in &args {
        match arg.as_str() {
            "--recursive" => options.recursive = true,
            "--all" => options.show_hidden = true,
            "--long" => options.long_format = true,
            "--help" | "-h" => { print_usage(); exit(0); },
            "--version" => { print_version!(); exit(0); },
            _ if arg.starts_with('-') => {
                for c in arg.chars().skip(1) {
                    match c {
                        'a' => options.show_hidden = true,
                        'l' => options.long_format = true,
                        'R' => options.recursive = true,
                        _ => {
                            safe_eprintln(format_args!("{}: unknow flag -{}", package_name!(), c));
                            exit(1);
                        }
                    }
                }
            },
            _ if !arg.starts_with('-') => path = arg,
            _ => {},
        }
    }

    if options.recursive {
        list_dir_recursive(path, options.show_hidden, options.long_format);
    } else {
        list_dir(path, options.show_hidden, options.long_format);
    }
}

fn list_dir<P: AsRef<Path>>(path: P, show_hidden: bool, long_format: bool) {
    let entries = match fs::read_dir(&path) {
        Ok(entries) => entries,
        Err(e) => {
            safe_eprintln(format_args!("{}: error read directory: {}", package_name!(), e));
            exit(1);
        }
    };

    let mut visible_entries: Vec<DirEntry> = Vec::new();

    for entry_result in entries {
        if let Ok(entry) = entry_result {
            let file_name = entry.file_name();
            let name_str = file_name.to_string_lossy();
            if !show_hidden && name_str.starts_with('.') {
                continue;
            }
            visible_entries.push(entry);
        }
    }

    // Sort
    visible_entries.sort_by_key(|e| e.file_name());

    for entry in &visible_entries {
        match entry.metadata() {
            Ok(meta) => {
                print_metadata(&entry, &meta, long_format);
            }
            Err(e) => {
                safe_eprintln(format_args!("{}: cannot access '{}': {}", package_name!(), entry.path().display(), e));
            }
        }
    }

    if !long_format && !&visible_entries.is_empty() {
        println!();
    }
}

fn list_dir_recursive<P: AsRef<Path>>(start_path: P, show_hidden: bool, long_format: bool) {
    let mut stack = vec![start_path.as_ref().to_path_buf()];

    while let Some(current_path) = stack.pop() {
        safe_println(format_args!("\n{}:", current_path.display()));

        let entries = match fs::read_dir(&current_path) {
            Ok(entries) => entries,
            Err(e) => {
                safe_eprintln(format_args!("{}: error reading {}: {}", package_name!(), current_path.display(), e));
                continue;
            }
        };

        let mut files = Vec::new();
        let mut dirs = Vec::new();

        for entry_result in entries {
            if let Ok(entry) = entry_result {
                let file_name = entry.file_name();
                let name_str = file_name.to_string_lossy();

                if !show_hidden && name_str.starts_with('.') {
                    continue;
                }

                match fs::symlink_metadata(entry.path()) {
                    Ok(meta) => {
                        print_metadata(&entry, &meta, long_format);
                        if meta.is_dir() {
                            dirs.push(entry.path());
                        } else {
                            files.push(entry);
                        }
                    }
                    Err(e) => {
                        safe_eprintln(format_args!("{}: cannot access {}: {}", package_name!(), entry.path().display(), e));
                    }
                }
            }
        }

        if !long_format {
            println!();
        }

        // Push child directories into the stack for next iteration
        dirs.sort(); // Optional: sort for predictable order
        stack.extend(dirs.into_iter().rev()); // Reverse to maintain depth-first order
    }
}

fn print_metadata(entry: &DirEntry, meta: &Metadata, long_format: bool) {
    let file_name = entry.file_name().to_string_lossy().to_string();
    let permissions = meta.permissions().mode();

    let (fg, _bg) = get_fg_bg_color(&entry, &meta);

    if long_format {
        let size = meta.len();
        let modified = meta.modified().unwrap_or(UNIX_EPOCH);
        let datetime: DateTime<Local> = DateTime::from(modified);

        let perms_str = format!(
            "{}[{}{}{}{}{}{}{}{}{}]",
            if meta.is_dir() { "d" } else { "f" },
            if (permissions & 0o400) != 0 { "r" } else { "-" },
            if (permissions & 0o200) != 0 { "w" } else { "-" },
            if (permissions & 0o100) != 0 { "x" } else { "-" },
            if (permissions & 0o040) != 0 { "r" } else { "-" },
            if (permissions & 0o020) != 0 { "w" } else { "-" },
            if (permissions & 0o010) != 0 { "x" } else { "-" },
            if (permissions & 0o004) != 0 { "r" } else { "-" },
            if (permissions & 0o002) != 0 { "w" } else { "-" },
            if (permissions & 0o001) != 0 { "x" } else { "-" }, 
        );

        use std::os::unix::fs::MetadataExt;
        let uid = meta.uid();
        let gid = meta.gid();

        let user = get_user_by_uid(uid)
            .map(|u| u.name().to_string_lossy().into_owned())
            .unwrap_or(uid.to_string());

        let group = get_group_by_gid(gid)
            .map(|g| g.name().to_string_lossy().into_owned())
            .unwrap_or(gid.to_string());

        safe_println(format_args!(
            "{} {:>8} {:<8} {:<8} {} \x1b[38;2;{};{};{}m{}{}\x1b[0m{}",
            perms_str,
            size,
            user,
            group,
            datetime.format("%Y-%m-%d %H:%M"),
            fg.0, fg.1, fg.2,
            file_name,
            "\x1b[0m",
            match meta.file_type().is_symlink() {
                true => match fs::read_link(entry.path()) {
                    Ok(target) => format!(" -> {}", target.display()),
                    Err(_) => format!(" -> error: No such file or directory")
                },
                false => String::from(""),
            }
        ));
    } else {
        safe_print(format_args!("\x1b[38;2;{};{};{}m{}{}\x1b[0m ", 
            fg.0, fg.1, fg.2,
            file_name,
            "\x1b[0m"
        ));
    }
}

fn print_usage() {
    safe_println(format_args!("Usage: {} [OPTIONS] [DIRECTORY NAME]", package_name!()));
    safe_println(format_args!("     a           List all files and directories"));
    safe_println(format_args!("     l           Show as list view"));
    safe_println(format_args!("     --help      Show help"));
    safe_println(format_args!("     --version   Show version"));
}

pub fn get_fg_bg_color(entry: &DirEntry, meta: &Metadata) -> ((u8, u8, u8), (u8, u8, u8)) {
    let binding = entry.file_name();
    let name = binding.to_string_lossy();
    let mode = meta.permissions().mode();
    let perms = mode & 0o777;
    let lower = name.to_lowercase();

    if meta.is_dir() {
        return ((0, 119, 255), (0, 0, 0)); // Dir
    }

    if meta.file_type().is_symlink() {
        return ((0, 255, 255), (0, 0, 0)); // Symlink
    }

    if mode & 0o111 != 0 {
        return ((0, 255, 0), (0, 0, 0)); // Executable
    }

    if perms == 0o777 {
        return ((255, 255, 255), (255, 0, 0)); // 777 (danger)
    }

    if name.starts_with('.') {
        return ((95, 95, 95), (0, 0, 0)); // Hidden file
    }

    if lower.ends_with(".tar") || lower.ends_with(".gz") || lower.ends_with(".zip") {
        return ((255, 0, 0), (0, 0, 0)); // Archive
    }

    if lower.ends_with(".jpg") || lower.ends_with(".png") || lower.ends_with(".gif") {
        return ((255, 0, 255), (0, 0, 0)); // Image
    }

    if lower.ends_with(".mp3") || lower.ends_with(".wav") {
        return ((0, 255, 255), (0, 0, 0)); // Audio
    }

    if lower.ends_with(".mp4") || lower.ends_with(".mkv") {
        return ((255, 0, 255), (0, 0, 0)); // Video
    }

    if lower.ends_with(".c") || lower.ends_with(".rs") || lower.ends_with(".sh") {
        return ((255, 255, 0), (0, 0, 0)); // Source code
    }

    ((210, 210, 210), (0, 0, 0))// Default
}
