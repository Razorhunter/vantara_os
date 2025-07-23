mod args;
mod process;
mod output;

use args::Options;
use process::get_processes;
use output::print_processes;
use vantara::{print_version, safe_println, package_name};

fn main() {
    let args = Options::parse();

    match args {
        _ if args.show_usage => { print_usage(); std::process::exit(0); },
        _ if args.show_version => { print_version!(); std::process::exit(0); }
        _ => {
            let processes = get_processes(&args);
            print_processes(&args, &processes);
        }
    }
}

fn print_usage() {
    safe_println(format_args!("Usage: {} -[OPTIONS]", package_name!()));
    safe_println(format_args!("     a           Show all processes"));
    safe_println(format_args!("     u           Show user-oriented format (USER, %CPU, %MEM, VSZ, RSS, START, TIME)"));
    safe_println(format_args!("     x           Show process without TTY(daemon / background)"));
    safe_println(format_args!("     e           Show all environment variable in CMD column"));
    safe_println(format_args!("     o           Custom input format (e.g, ps -o pid,cmd)"));
    safe_println(format_args!("     --help      Show help"));
    safe_println(format_args!("     --version   Show version"));
}
