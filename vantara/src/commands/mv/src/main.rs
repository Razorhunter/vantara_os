use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{exit};
use vantara::{package_name, expand_wildcards, safe_eprintln, safe_println, print_version, confirm};

struct Options {
    recursive: bool,
    force_ignore: bool,
    no_overwrite: bool,
    interactive: bool,
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let mut options = Options {
        recursive: false,
        force_ignore: false,
        no_overwrite: true,
        interactive: true,
    };

    //Check for empty arguments
    if args.is_empty() {
        safe_println(format_args!("{}: no arguments", package_name!()));
        print_usage();
        exit(1);
    }

    let mut paths: Vec<String> = Vec::new();

    for arg in args {
        match arg.as_str() {
            "--recursive" => options.recursive = true,
            "--force" => options.force_ignore = true,
            "--no-overwrite" => options.no_overwrite = true,
            "--non-interactive" => options.interactive = false,
            "--help" => { print_usage(); return; },
            "--version" => { print_version!(); return; },
            _ if arg.starts_with('-') => {
                for c in arg.chars().skip(1) {
                    match c {
                        'r' => options.recursive = true,
                        'f' => options.force_ignore = true,
                        'n' => options.no_overwrite = true,
                        _ => {
                            safe_eprintln(format_args!("{}: unknown flag -{}", package_name!(), c));
                            exit(1);
                        }
                    }
                }
            },
            _ if !arg.starts_with('-') => paths.push(arg.clone()),
            _ => {}
        }
    }

    if options.force_ignore {
        options.no_overwrite = false;
    }

    //Checking for empty path
    if paths.is_empty() {
        safe_println(format_args!("{}: please specify directory or filename", package_name!()));
        print_usage();
        exit(1);
    }

    paths = expand_wildcards(&paths);

    let dest: &Path = Path::new(paths.last().unwrap());
    let mut src_paths: Vec<PathBuf> = Vec::new();

    //Grouping the path into source and destination
    for path in &paths[..paths.len() - 1] {
        src_paths.push(PathBuf::from(path));
    }

    let multiple_sources = src_paths.len() > 1;

    //Show prompt message
    if options.interactive {
        if src_paths.len() > 1 {
            for src_path in &src_paths {
                safe_println(format_args!("{}", src_path.display()));
            }
            safe_println(format_args!("{} items match your request", src_paths.len()));
            if !confirm(&format!("Move all these files/directories to '{}'?", &dest.display())) {
                safe_println(format_args!("{}: aborted.", package_name!()));
                return;
            }
        } else if src_paths.len() == 1 {
            if !confirm(&format!("Move '{}' to '{}?", src_paths[0].display(), &dest.display())) {
                safe_println(format_args!("{}: skipped '{}'", package_name!(), src_paths[0].display()));
                return;
            }
        } else {
            safe_println(format_args!("{}: error moving files or directories", package_name!()));
            print_usage();
            exit(1);
        }
    }

    for src_str in src_paths {
        let src = Path::new(&src_str);

        //Checking source exist or not
        if !src.exists() {
            safe_println(format_args!("{}: cannot stat '{}': No such file or directory", package_name!(), src.display()));
            exit(1);
        }

        if src.is_file() {
            if dest.is_file() {
                //This block code just to do verification before actual copy
                //Do not allow multiple source write to single file
                if multiple_sources {
                    safe_println(format_args!("{}: cannot overwrite multiple files into single file", package_name!()));
                    exit(1);
                }

                //Checking filename is same or not (basically in case of same location with same filename input by user)
                if src.display().to_string() == dest.display().to_string() && !options.force_ignore {
                    safe_println(format_args!("{}: '{}' and '{}' is the same file", package_name!(), src.display(), dest.display()));
                    if options.no_overwrite {
                        if !confirm(&format!("Move anyway?")) {
                            safe_println(format_args!("{}: aborted", package_name!()));
                            exit(0);
                        }
                    }
                }

                //Check in destination have a same filename or not (maybe different location that got same filename)
                if dest.exists() && options.no_overwrite {
                    safe_println(format_args!("{}: '{}' already exists", package_name!(), dest.display()));
                    if !confirm(&format!("Move anyway?")) {
                        safe_println(format_args!("{}: aborted", package_name!()));
                        exit(0);
                    }
                }
            } else if dest.is_dir() {
                //Check if inside directory have a file with same filename. If yes, ask either to overwrite of skip
                let target_path = dest.join(src.file_name().unwrap());

                if target_path.exists() && !options.force_ignore {
                    safe_println(format_args!("There is file with same filename exist in directory '{}'", dest.display()));
                    if !confirm(&format!("File will be overwrite and cannot be undone. Move anyway?")) {
                        safe_println(format_args!("{}: skipped '{}'", package_name!(), src.display()));
                        continue;
                    }
                }
            }

            //If everything just fine, we do actual copy file here
            if let Err(e) = copy_file(src, &dest, !options.force_ignore) {
                safe_eprintln(format_args!("{}: error moving '{}' to '{}': {}", package_name!(), src.display(), dest.display(), e));
                exit(1);
            } else {
                let _ = fs::remove_file(src);
            }
        } else if src.is_dir() {
            //Checking for recursive when copy directory
            if !options.recursive {
                safe_println(format_args!("{}: omitting directory '{}' because -r not specified", package_name!(), src.display()));
                exit(1);
            }

            //Cannot write folder to file
            if dest.is_file() {
                safe_println(format_args!("{}: cannot overwrite non-directory '{}' with directory '{}'", package_name!(), src.display(), dest.display()));
                exit(1);
            }

            //If everything just fine, we do actual copy directory here
            let src_dir_name = match src.file_name() {
                Some(name) => name,
                None => {
                    safe_println(format_args!("{}: invalid source directory", package_name!()));
                    exit(1);
                }
            };

            let final_dest = if dest.exists() && dest.is_dir() {
                dest.join(src_dir_name)
            } else {
                dest.to_path_buf()
            };

            if let Err(e) = copy_dir_recursive(src, &final_dest, !options.force_ignore) {
                safe_eprintln(format_args!("{}: error moving '{}' to '{}': {}", package_name!(), src.display(), dest.display(), e));
                exit(1);
            } else {
                let _ = fs::remove_dir_all(src);
            }
        }
    }
}

fn copy_file(src: &Path, dest: &Path, force: bool) -> io::Result<()> {
    let final_dest = if dest.is_dir() {
        let original_name = src.file_name().unwrap().to_str().unwrap();

        if dest.join(original_name).exists() && force {
            generate_unique_name(dest, original_name)
        } else {
            dest.join(original_name)
        }
    } else {
        if dest.exists() && force {
            generate_unique_name(dest.parent().unwrap(), dest.file_name().unwrap().to_str().unwrap())
        } else {
            dest.to_path_buf()
        }
    };

    fs::copy(src, final_dest)?;
    Ok(())
}

fn copy_dir_recursive(src: &Path, dest: &Path, force: bool) -> io::Result<()> {
    if !dest.exists() {
        fs::create_dir_all(&dest)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());

        if file_type.is_dir() {
            copy_dir_recursive(&src_path, &dest_path, force)?;
        } else {
            copy_file(&src_path, &dest_path, force)?;
        }
    }

    Ok(())
}

fn generate_unique_name(dest_dir: &Path, file_name: &str) -> PathBuf {
    let mut candidate = dest_dir.join(file_name);

    if !candidate.exists() {
        return candidate;
    }

    let mut count = 1;
    let path = Path::new(file_name);
    let stem = path.file_stem().unwrap().to_string_lossy();
    let ext = path.extension().map(|e| e.to_string_lossy()).unwrap_or_default();

    loop {
        let new_name = if ext.is_empty() {
            format!("{}.{}", stem, count)
        } else {
            format!("{}.{}.{}", stem, count, ext)
        };

        candidate = dest_dir.join(new_name);
        if !candidate.exists() {
            break;
        }
        count += 1;
    }

    candidate
}

fn print_usage() {
    safe_println(format_args!("Usage: {} -[OPTIONS] [SOURCES] [DESTINATION]", package_name!()));
    safe_println(format_args!("     f | --force             Moving without confirmation prompt"));
    safe_println(format_args!("     n | --no-overwrite      Keep the same file name in the destination if exist"));
    safe_println(format_args!("     --non-interactive       Skipped move confirmation message"));
    safe_println(format_args!("     --help                  Show help"));
    safe_println(format_args!("     --version               Show version"));
}
