use std::env;
use std::path::Path;
use std::fs::{metadata, symlink_metadata};
use nix::unistd::{chown, Gid, Uid};
use users::{get_user_by_name, get_group_by_name};
use std::process::exit;
use walkdir::WalkDir;
use vantara::{print_version, safe_println, safe_eprintln, package_name};

#[derive(Default, Debug)]
struct Options {
    recursive: bool,
    verbose: bool,
    silent: bool,
    dereference: bool,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut options = Options::default();
    let mut owner_spec = None;
    let mut paths: Vec<String> = Vec::new();

    let mut arg_iter = args[1..].iter();

    while let Some(arg) = arg_iter.next() {
        if arg == "--version" {
            print_version!();
            exit(0);
        } else if arg == "--help" {
            print_usage();
            exit(0);
        } else if arg.starts_with('-') && arg.len() > 1 {
            for c in arg.chars().skip(1) {
                match c {
                    'R' => options.recursive = true,
                    'v' => options.verbose = true,
                    'f' => options.silent = true,
                    'L' => options.dereference = true,
                    'P' => options.dereference = false,
                    _ => {
                        safe_eprintln(format_args!("{}: unknown flag: -{}", package_name!(), c));
                        exit(1);
                    }
                }
            }
        } else if owner_spec.is_none() {
            owner_spec = Some(arg.clone());
        } else {
            paths.push(arg.clone());
        }
    }

    if owner_spec.is_none() || paths.is_empty() {
        safe_eprintln(format_args!("{}: usage: rchown [-RvfLP] user:group FILE...", package_name!()));
        exit(1);
    }

    let (uid, gid) = parse_user_group(&owner_spec.unwrap());

    for path in paths {
        let target_path = Path::new(&path);
        if options.recursive && target_path.is_dir() {
            for entry in WalkDir::new(target_path).into_iter().filter_map(Result::ok) {
                apply_chown(entry.path(), uid, gid, &options);
            }
        } else {
            apply_chown(target_path, uid, gid, &options);
        }
    }
}

fn parse_user_group(input: &str) -> (Option<Uid>, Option<Gid>) {
    let parts: Vec<&str> = input.split(':').collect();

    let (user, group) = match parts.len() {
        1 => {
            if input.contains(':') {
                (None, Some(parts[1]))
            } else {
                (Some(parts[0]), None)
            }
        }
        2 => (Some(parts[0]), Some(parts[1])),
        _ => (None, None),
    };

    let uid = user.and_then(|u| get_user_by_name(u).map(|u| Uid::from_raw(u.uid())));
    let gid = group.and_then(|g| get_group_by_name(g).map(|g| Gid::from_raw(g.gid())));
    (uid, gid)
}

fn apply_chown(path: &Path, uid: Option<Uid>, gid: Option<Gid>, options: &Options) {
    let meta = if options.dereference {
        metadata(path)
    } else {
        symlink_metadata(path)
    };

    match meta {
        Ok(_) => {
            if let Err(e) = chown(path, uid, gid) {
                if !options.silent {
                    safe_eprintln(format_args!("{}: failed to chown {}: {}", package_name!(), path.display(), e));
                }
            } else if options.verbose {
                safe_println(format_args!("{}: changed ownership of {}", package_name!(), path.display()));
            }
        }
        Err(e) => {
            if !options.silent {
                safe_eprintln(format_args!("{}: cannot access {}: {}", package_name!(), path.display(), e));
            }
        }
    }
}

fn print_usage() {
    safe_println(format_args!("Usage: {} [OPTIONS] [MODE] [TARGET]", package_name!()));
}
