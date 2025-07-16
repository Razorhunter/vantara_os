use chrono::Local;
use vantara::{safe_println, safe_eprintln};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        // Tiada argumen: papar tarikh & masa sekarang
        let now = Local::now();
        safe_println(format_args!("{}", now.format("%a %b %e %Y %T %Z")));
    } else {
        safe_eprintln(format_args!("Usage: {} [no arguments]", args[0]));
    }
}
