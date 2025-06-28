use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Seek, SeekFrom};
use std::env;

fn main() -> io::Result<()> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // Tangkap SIGINT (Ctrl+C)
    ctrlc::set_handler(move || {
        println!("\nReceived Ctrl+C. Exiting...");
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl+C handler");

    // Arg parsing manual (boleh ganti dengan versi kau sendiri)
    let args: Vec<String> = env::args().collect();
    let mut lines = 10;
    let mut follow = false;
    let mut file_path = String::new();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-n" => {
                lines = args[i + 1].parse().unwrap_or(10);
                i += 1;
            }
            "-f" => {
                follow = true;
            }
            _ => {
                file_path = args[i].clone();
            }
        }
        i += 1;
    }

    let mut file = File::open(&file_path)?;
    let mut reader = BufReader::new(file.try_clone()?);
    let all_lines: Vec<String> = reader.lines().filter_map(Result::ok).collect();
    let start = if all_lines.len() > lines {
        all_lines.len() - lines
    } else {
        0
    };
    for line in &all_lines[start..] {
        println!("{}", line);
    }

    if follow {
        let mut pos = file.metadata()?.len();
        while running.load(Ordering::SeqCst) {
            thread::sleep(Duration::from_secs(1));
            file = File::open(&file_path)?;
            file.seek(SeekFrom::Start(pos))?;
            let mut reader = BufReader::new(file.try_clone()?);

            for line in reader.lines() {
                if let Ok(l) = line {
                    println!("{}", l);
                }
            }
            pos = file.metadata()?.len();
        }
    }

    Ok(())
}
