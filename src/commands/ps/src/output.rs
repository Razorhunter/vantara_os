use crate::{args::Options, process::ProcInfo};
use procfs::Current;

fn read_mem_percent(rss_kb: u64) -> f64 {
    let mem_total_kb = procfs::Meminfo::current().unwrap().mem_total;
    (rss_kb as f64 / mem_total_kb as f64) * 100.0
}

pub fn print_processes(_args: &Options, processes: &[ProcInfo]) {
    println!("{:<8} {:<10} {:<5} {:<9} {:>5} {:>5} {:>10} {:>10} {:>13} {}", 
        "PID", "USER", "STAT", "TTY", "%CPU", "%MEM", "VSZ", "RSS", "TIME", "COMMAND");

    for proc in processes {
        let mem_percent = read_mem_percent(proc.rss);
        println!("{:<8} {:<10} {:<5} {:<9} {:>5.1} {:>5.1} {:>10} {:>10} {:>13} {}", 
            proc.pid, proc.user, proc.stat, proc.tty,proc.cpu_percent, mem_percent, proc.vsz, proc.rss, proc.time, proc.cmd);
    }
}
