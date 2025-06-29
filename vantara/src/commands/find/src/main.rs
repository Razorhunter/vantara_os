use std::env;
use std::process::exit;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::{DirEntry, WalkDir};
use vantara::{safe_println, safe_eprintln, package_name, print_version};
use glob::Pattern;
use std::time::{SystemTime, Duration};
use std::fs;

#[derive(Debug)]
struct Options {
    name: Option<Pattern>,
    file_type: Option<char>,
    maxdepth: Option<usize>,
    exec: Option<Vec<String>>,
    mtime: Option<(char, i64)>,
}

enum Expr {
    Name(Pattern),
    MTime(char, i64),
    Type(char),
    Not(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    //Check for empty arguments
    if args.is_empty() {
        safe_println(format_args!("{}: no arguments", package_name!()));
        print_usage();
        exit(1);
    }

    let mut options = Options {
        name: None,
        file_type: None,
        maxdepth: None,
        exec: None,
        mtime: None,
    };

    let mut root_path = PathBuf::new();
    let mut i = 0;

    while i < args.len() {
        match args[i].as_str() {
            "--help" => { print_usage(); exit(0); },
            "--version" => { print_version!(); exit(0); },
            "-name" => {
                i += 1;
                if let Some(pattern_str) = args.get(i) {
                    let has_wildcard = pattern_str.contains('*') || pattern_str.contains('?') || pattern_str.contains('[');

                    let final_pattern = if has_wildcard {
                        pattern_str.to_string()
                    } else {
                        Pattern::escape(pattern_str)
                    };

                    match Pattern::new(&final_pattern) {
                        Ok(pat) => options.name = Some(pat),
                        Err(e) => {
                            safe_eprintln(format_args!("{}: invalid pattern '{}': {}", package_name!(), pattern_str, e));
                            return;
                        }
                    }
                }
            },
            "-type" => {
                i += 1;
                if let Some(t) = args.get(i) {
                    options.file_type = t.chars().next();
                }
            },
            "-maxdepth" => {
                i += 1;
                if let Some(d) = args.get(i) {
                    options.maxdepth = d.parse().ok();
                }
            },
            "-exec" => {
                i += 1;
                let mut cmd_exec = Vec::new();
                while i < args.len() && args[i] != ";" {
                    cmd_exec.push(args[i].clone());
                    i += 1;
                }
                options.exec = Some(cmd_exec);
            },
            "-mtime" => {
                i += 1;
                if let Some(mtime_str) = args.get(i) {
                    let (op, value_str) = match mtime_str.chars().next() {
                        Some(c @ ('+' | '-')) => (c, &mtime_str[1..]),
                        _ => ('=', mtime_str.as_str()),
                    };
                    if let Ok(days) = value_str.parse::<i64>() {
                        options.mtime = Some((op, days));
                    } else {
                        safe_eprintln(format_args!("{}: Invalid -mtime value: {}", package_name!(), mtime_str));
                        return;
                    }
                }
            },
            p if root_path.as_os_str().is_empty() => {
                root_path = PathBuf::from(p);
            },
            _ => {}
        }

        i += 1;
    }

    let walker = WalkDir::new(&root_path)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            if let Some(max) = options.maxdepth {
                e.depth() <= max
            } else {
                true
            }
        });

    for entry in walker.filter_map(Result::ok) {
        if is_matching(&entry, &options) {
            safe_println(format_args!("{}", entry.path().display()));

            if let Some(ref cmd_exec) = options.exec {
                run_exec(cmd_exec, entry.path());
            }
        }
    }
}

fn is_matching(entry: &DirEntry, options: &Options) -> bool {
    let path = entry.path();

    if let Some(ref name) = options.name {
        if let Some(fname) = path.file_name() {
            if !name.matches(&fname.to_string_lossy()) {
                return false;
            }
        } else {
            return false;
        }
    }

    if let Some(t) = options.file_type {
        match t {
            'f' if !path.is_file() => return false,
            'd' if !path.is_dir() => return false,
            _ => {}
        }
    }

    if let Some((op, days)) = options.mtime {
        if let Ok(metadata) = fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = modified.elapsed() {
                    let file_age_days = elapsed.as_secs() as i64 / 86400;

                    let matched = match op {
                        '+' => file_age_days > days,
                        '-' => file_age_days < days,
                        '=' => file_age_days == days,
                        _ => false,
                    };

                    if !matched {
                        return false;
                    }
                }
            }
        }
    }

    true
}

fn run_exec(cmd_template: &[String], file_path: &Path) {
    let mut command_parts = cmd_template.to_vec();
    for part in &mut command_parts {
        if part == "{}" {
            *part = file_path.to_string_lossy().to_string();
        }
    }

    if let Some((cmd, args)) = command_parts.split_first() {
        let status = Command::new(cmd).args(args).status();
        if let Err(e) = status {
            eprintln!("Failed to execute {:?}: {}", cmd, e);
        }
    }
}

fn print_usage() {
    safe_println(format_args!("Usage: {} [ROOT_PATH] -[OPTIONS]", package_name!()));
    safe_println(format_args!("     [ROOT_PATH]             Directory name to begin find. Enter directory name of dot (.) for current directory"));
    safe_println(format_args!("     name [search_name]      Search by name or using regex"));
    safe_println(format_args!("     type [f/d]              Filter f = file, d = directory"));
    safe_println(format_args!("     maxdepth                How deep directory depth control"));
    safe_println(format_args!("     exec                    Run commands on files"));
    safe_println(format_args!("     --help                  Show help"));
    safe_println(format_args!("     --version               Show version"));
}
