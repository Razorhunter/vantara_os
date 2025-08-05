use std::env;
use std::process::exit;
use std::path::Path;
use vantara::{safe_println, safe_eprintln, package_name, print_version};
use glob::Pattern;
use std::fs;

enum Expr {
    Name(Pattern),
    Type(char),
    MTime(char, i64),
    Size(char, u64, SizeUnit),
    Not(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Exec(Vec<String>),
}

#[derive(PartialEq)]
enum SizeUnit {
    Bytes,
    Kilobytes,
    Megabytes,
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        safe_println(format_args!("{}: find <path> [expression]", package_name!()));
        exit(1);
    }

    let mut root = ".";
    let mut maxdepth: Option<usize> = None;

    let mut i = 0;
    if !args[i].starts_with('-') {
        root = &args[i];
        i += 1;
    }

    let mut filtered_args = vec![];
    while i < args.len() {
        if args[i] == "-maxdepth" {
            i += 1;
            if i < args.len() {
                maxdepth = args[i].parse::<usize>().ok();
                i += 1;
            } else {
                safe_eprintln(format_args!("{}: missing value for -maxdepth", package_name!()));
                return;
            }
        } else {
            filtered_args.push(args[i].clone());
            i += 1;
        }
    }

    if let Some(expr) = parse_expr(&filtered_args) {
        walk_dir(Path::new(root), &expr, 0, maxdepth);
    } else {
        safe_eprintln(format_args!("{}: invalid expression", package_name!()));
    }
}


fn walk_dir(path: &Path, expr: &Expr, depth: usize, maxdepth: Option<usize>) {
    if let Some(max) = maxdepth {
        if depth > max {
            return;
        }
    }

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if evaluate_expr(expr, &path) {
                println!("{}", path.display());
                if let Expr::Exec(cmd) = expr {
                    run_exec(cmd, &path);
                }
            }
            if path.is_dir() {
                walk_dir(&path, expr, depth + 1, maxdepth);
            }
        }
    }
}

fn parse_expr(args: &[String]) -> Option<Expr> {
    let mut i = 0;

    fn next_expr(args: &[String], i: &mut usize) -> Option<Expr> {
        if *i >= args.len() {
            return None;
        }

        match args[*i].as_str() {
            "--help" => { print_usage(); exit(1); },
            "--version" => { print_version!(); exit(1); },
            "-name" => {
                *i += 1;
                if *i < args.len() {
                    let pattern = &args[*i];
                    let has_wildcard = pattern.contains('*') || pattern.contains('?') || pattern.contains('[');
                    let final_pat = if has_wildcard {
                        pattern.clone()
                    } else {
                        Pattern::escape(pattern)
                    };
                    *i += 1;
                    Some(Expr::Name(Pattern::new(&final_pat).ok()?))
                } else {
                    None
                }
            },
            "-type" => {
                *i += 1;
                if *i < args.len() {
                    let t = args[*i].chars().next()?;
                    *i += 1;
                    Some(Expr::Type(t))
                } else {
                    None
                }
            },
            "-mtime" => {
                *i += 1;
                if *i < args.len() {
                    let val = &args[*i];
                    let (op, val_str) = match val.chars().next() {
                        Some(c @ ('+' | '-')) => (c, &val[1..]),
                        _ => ('=', val.as_str()),
                    };
                    let days = val_str.parse::<i64>().ok()?;
                    *i += 1;
                    Some(Expr::MTime(op, days))
                } else {
                    None
                }
            },
            "!" => {
                *i += 1;
                let inner = next_expr(args, i)?;
                Some(Expr::Not(Box::new(inner)))
            },
            "-size" => {
                *i += 1;
                if *i < args.len() {
                    let val = &args[*i];
                    let (op, rest) = match val.chars().next() {
                        Some(c @ ('+' | '-' | '=')) => (c, &val[1..]),
                        _ => ('=', val.as_str()),
                    };

                    let unit = match rest.chars().last() {
                        Some('k') | Some('K') => SizeUnit::Kilobytes,
                        Some('M') | Some('m') => SizeUnit::Megabytes,
                        Some('c') => SizeUnit::Bytes,
                        _ => SizeUnit::Bytes,
                    };

                    let num_str = if unit == SizeUnit::Bytes {
                        rest
                    } else {
                        &rest[..rest.len()-1]
                    };

                    if let Ok(size) = num_str.parse::<u64>() {
                        *i += 1;
                        Some(Expr::Size(op, size, unit))
                    } else {
                        None
                    }
                } else {
                    None
                }
            },
            "-exec" => {
                *i += 1;
                let mut cmd = Vec::new();
                while *i < args.len() {
                    if args[*i] == ";" || args[*i] == "\\;" {
                        *i += 1;
                        break;
                    }
                    cmd.push(args[*i].clone());
                    *i += 1;
                }
                Some(Expr::Exec(cmd))
            },
            _ => None,
        }
    }

    let mut expr = next_expr(args, &mut i)?;
    while i < args.len() {
        match args[i].as_str() {
            "-or" => {
                i += 1;
                let rhs = next_expr(args, &mut i)?;
                expr = Expr::Or(Box::new(expr), Box::new(rhs));
            }
            "-and" => {
                i += 1;
                let rhs = next_expr(args, &mut i)?;
                expr = Expr::And(Box::new(expr), Box::new(rhs));
            }
            _ => {
                let rhs = next_expr(args, &mut i)?;
                expr = Expr::And(Box::new(expr), Box::new(rhs)); // default: AND
            }
        }
    }

    Some(expr)
}

fn evaluate_expr(expr: &Expr, path: &Path) -> bool {
    match expr {
        Expr::Name(pat) => {
            path.file_name()
                .map(|n| pat.matches(&n.to_string_lossy()))
                .unwrap_or(false)
        },
        Expr::Type(t) => {
            fs::metadata(path).map(|m| {
                match t {
                    'f' => m.is_file(),
                    'd' => m.is_dir(),
                    _ => false,
                }
            }).unwrap_or(false)
        },
        Expr::MTime(op, days) => {
            if let Ok(meta) = fs::metadata(path) {
                if let Ok(modified) = meta.modified() {
                    if let Ok(elapsed) = modified.elapsed() {
                        let age = elapsed.as_secs() as i64 / 86400;
                        return match op {
                            '+' => age > *days,
                            '-' => age < *days,
                            '=' => age == *days,
                            _ => false,
                        };
                    }
                }
            }
            false
        },
        Expr::Size(op, size, unit) => {
            if let Ok(meta) = fs::metadata(path) {
                let file_size = meta.len(); // dalam bytes
                let cmp_size = match unit {
                    SizeUnit::Bytes => *size,
                    SizeUnit::Kilobytes => *size * 1024,
                    SizeUnit::Megabytes => *size * 1024 * 1024,
                };

                match op {
                    '+' => file_size > cmp_size,
                    '-' => file_size < cmp_size,
                    '=' => file_size == cmp_size,
                    _ => false,
                }
            } else {
                false
            }
        },
        Expr::Not(inner) => !evaluate_expr(inner, path),
        Expr::And(a, b) => evaluate_expr(a, path) && evaluate_expr(b, path),
        Expr::Or(a, b) => evaluate_expr(a, path) || evaluate_expr(b, path),
        &Expr::Exec(_) => {
            safe_eprintln(format_args!("{}: invalid", package_name!()));
            exit(1);
        },
    }
}

fn run_exec(cmd: &[String], path: &Path) {
    use std::process::Command;

    let mut real_cmd: Vec<String> = vec![];

    for arg in cmd {
        if arg == "{}" {
            real_cmd.push(path.to_string_lossy().into_owned());
        } else {
            real_cmd.push(arg.clone());
        }
    }

    if let Some((prog, args)) = real_cmd.split_first() {
        let _ = Command::new(prog)
            .args(args)
            .status();
    }
}

fn print_usage() {
    safe_println(format_args!("Usage: {} -[OPTIONS]", package_name!()));
    safe_println(format_args!("     name [search_name]      Search by name or using regex"));
    safe_println(format_args!("     type [f/d]              Filter f = file, d = directory"));
    safe_println(format_args!("     maxdepth                How deep directory depth control"));
    safe_println(format_args!("     exec                    Run commands on files"));
    safe_println(format_args!("     --help                  Show help"));
    safe_println(format_args!("     --version               Show version"));
}
