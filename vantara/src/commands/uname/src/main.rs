use libc::utsname;
use std::env;
use std::ffi::CStr;
use vantara::{safe_eprintln, safe_println, package_name, print_version};
use std::process::exit;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect(); // skip program name

    let (sysname, nodename, release, version, machine) = match get_uname() {
        Ok(info) => info,
        Err(e) => {
            safe_eprintln(format_args!("{}", e));
            exit(1);
        }
    };

    if args.is_empty() {
        safe_println(format_args!("{}", sysname));
        return;
    }

    let mut output = Vec::new();
    for arg in args {
        match arg.as_str() {
            "--help" => { print_usage(); exit(0); },
            "--version" => { print_version!(); exit(0) },
            _ if arg.starts_with('-') => {
                for ch in arg.chars().skip(1) {
                    match ch {
                        's' => output.push(sysname.clone()),
                        'n' => output.push(nodename.clone()),
                        'r' => output.push(release.clone()),
                        'v' => output.push(version.clone()),
                        'm' | 'p' | 'i' => output.push(machine.clone()),
                        'a' => {
                            output = vec![
                                sysname.clone(),
                                nodename.clone(),
                                release.clone(),
                                version.clone(),
                                machine.clone(),
                            ];
                            break;
                        }
                        _ => {
                            safe_eprintln(format_args!("{}: unknown flag -{}", package_name!(), ch));
                            exit(1);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    safe_println(format_args!("{}", output.join(" ")));
}

fn get_uname() -> Result<(String, String, String, String, String), String> {
    unsafe {
        let mut uname_data: utsname = std::mem::zeroed();

        if libc::uname(&mut uname_data) != 0 {
            return Err(format!("{}: failed to call uname", package_name!()));
        }

        Ok((
            CStr::from_ptr(uname_data.sysname.as_ptr()).to_string_lossy().to_string(),
            CStr::from_ptr(uname_data.nodename.as_ptr()).to_string_lossy().to_string(),
            CStr::from_ptr(uname_data.release.as_ptr()).to_string_lossy().to_string(),
            CStr::from_ptr(uname_data.version.as_ptr()).to_string_lossy().to_string(),
            CStr::from_ptr(uname_data.machine.as_ptr()).to_string_lossy().to_string(),
        ))
    }
}

fn print_usage() {
    safe_println(format_args!("Usage: {} -[OPTIONS]", package_name!()));
    safe_println(format_args!("     s           Print kernel name"));
    safe_println(format_args!("     n           Print network node hostname"));
    safe_println(format_args!("     r           Print kernel release"));
    safe_println(format_args!("     v           Print kernel version"));
    safe_println(format_args!("     m           Print machine hardware name"));
    safe_println(format_args!("     a           Print all the information above"));
}
