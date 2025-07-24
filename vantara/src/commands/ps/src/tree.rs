use std::collections::HashMap;
use crate::process::ProcInfo;

pub fn build_process_tree(procs: &[ProcInfo]) -> HashMap<i32, Vec<&ProcInfo>> {
    let mut tree: HashMap<i32, Vec<&ProcInfo>> = HashMap::new();
    for proc in procs {
        tree.entry(proc.ppid).or_default().push(proc);
    }
    tree
}

pub fn print_process_tree(tree: &HashMap<i32, Vec<&ProcInfo>>, pid: i32, level: usize, prefix: String, is_last: bool) {
    if let Some(children) = tree.get(&pid) {
        let len = children.len();
        for (i, proc) in children.iter().enumerate() {
            let is_last_child = i == len - 1;

            // ASCII line parts
            let branch = if level == 0 {
                "".to_string()
            } else if is_last {
                format!("{}└─ ", prefix)
            } else {
                format!("{}├─ ", prefix)
            };

            // Print this process
            println!("{}{} {} {}", branch, proc.pid, proc.user, proc.cmd);

            // Prefix for next level
            let new_prefix = if level == 0 {
                "".to_string()
            } else if is_last {
                format!("{}   ", prefix)
            } else {
                format!("{}│  ", prefix)
            };

            // Recursive call
            print_process_tree(tree, proc.pid, level + 1, new_prefix, is_last_child);
        }
    }
}