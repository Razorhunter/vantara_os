use std::env;
use std::fs::read_to_string;
use std::process::exit;
use vantara::{safe_println, safe_eprintln, package_name, print_version};

const DEFAULT_MEMORY_PROC: &str = "/proc/meminfo";
enum Unit {
    B,
    K,
    M,
    G,
    Human,
}

impl Unit {
    fn factor(&self) -> u64 {
        match self {
            Unit::B => 1,
            Unit::K => 1024,
            Unit::M => 1024 * 1024,
            Unit::G => 1024 * 1024 * 1024,
            Unit::Human => 1024, // for human-readable, we scale dynamically
        }
    }
}

fn main() {
    let (unit, show_total) = parse_args();
    let meminfo = read_meminfo();
    display(&meminfo, unit, show_total);
}


fn parse_args() -> (Unit, bool) {
    let args: Vec<String> = env::args().collect();

    if args.is_empty() {
        safe_println(format_args!("{}: no arguments", package_name!()));
        print_usage();
        exit(1);
    }

    let mut unit = Unit::K;
    let mut show_total = false;

    for arg in &args[1..] {
        match arg.as_str() {
            "-b" => unit = Unit::B,
            "-k" => unit = Unit::K,
            "-m" => unit = Unit::M,
            "-g" => unit = Unit::G,
            "-h" => unit = Unit::Human,
            "-t" => show_total = true,
            "--help" => {
                print_usage();
                exit(0);
            },
            "--version" => {
                print_version!();
                exit(0);
            }
            _ => {
                safe_eprintln(format_args!("{}: unknown option: {}", package_name!(), arg));
                print_usage();
                exit(1);
            }
        }
    }

    (unit, show_total)
}

fn read_meminfo() -> std::collections::HashMap<String, u64> {
    let content = read_to_string(DEFAULT_MEMORY_PROC).expect(&format!("{}: Failed to read {}", package_name!(), DEFAULT_MEMORY_PROC));
    let mut meminfo = std::collections::HashMap::new();

    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let key = parts[0].trim_end_matches(':').to_string();
            let value: u64 = parts[1].parse().unwrap_or(0);
            meminfo.insert(key, value); // value in kB
        }
    }

    meminfo
}

fn display(meminfo: &std::collections::HashMap<String, u64>, unit: Unit, show_total: bool) {
    let to_unit = |kb: u64| match unit {
        Unit::Human => human_readable(kb * 1024),
        _ => format!("{}", (kb * 1024) / unit.factor()),
    };

    let mem_total = meminfo.get("MemTotal").copied().unwrap_or(0);
    let mem_free = meminfo.get("MemFree").copied().unwrap_or(0);
    let buffers = meminfo.get("Buffers").copied().unwrap_or(0);
    let cached = meminfo.get("Cached").copied().unwrap_or(0);
    let sreclaimable = meminfo.get("SReclaimable").copied().unwrap_or(0);
    let shmem = meminfo.get("Shmem").copied().unwrap_or(0);

    let used = mem_total - mem_free - buffers - cached;
    let _available = meminfo.get("MemAvailable").copied().unwrap_or(0);

    safe_println(format_args!("{:<10} {:>10} {:>10} {:>10} {:>10} {:>10}",
             "Type", "Total", "Used", "Free", "Shared", "Buff/Cache"));
    safe_println(format_args!("{:<10} {:>10} {:>10} {:>10} {:>10} {:>10}",
             "Mem:",
             to_unit(mem_total),
             to_unit(used),
             to_unit(mem_free),
             to_unit(shmem),
             to_unit(buffers + cached + sreclaimable)));

    if show_total {
        let swap_total = meminfo.get("SwapTotal").copied().unwrap_or(0);
        let swap_free = meminfo.get("SwapFree").copied().unwrap_or(0);
        let swap_used = swap_total - swap_free;

        safe_println(format_args!("{:<10} {:>10} {:>10} {:>10}",
                 "Swap:",
                 to_unit(swap_total),
                 to_unit(swap_used),
                 to_unit(swap_free)));
    }
}

fn human_readable(bytes: u64) -> String {
    let units = ["B", "K", "M", "G", "T"];
    let mut size = bytes as f64;
    let mut unit = 0;

    while size > 1024.0 && unit < units.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }

    format!("{:.1}{}", size, units[unit])
}

fn print_usage() {
    safe_println(format_args!("Usage: {} [OPTIONS]", package_name!()));
    safe_println(format_args!("Options:"));
    safe_println(format_args!("     -b              Show output in bytes"));
    safe_println(format_args!("     -k              Show output in kilobytes (default)"));
    safe_println(format_args!("     -m              Show output in megabytes"));
    safe_println(format_args!("     -g              Show output in gigabytes"));
    safe_println(format_args!("     -h              Show output in human-readable format"));
    safe_println(format_args!("     -t              Show total memory line"));
    safe_println(format_args!("     --help          Show this help"));
    safe_println(format_args!("     --version       Show version"));
}

