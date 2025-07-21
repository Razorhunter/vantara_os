mod args;
mod process;
mod output;

use args::Options;
use process::get_processes;
use output::print_processes;

fn main() {
    let args = Options::parse();
    let processes = get_processes(&args);
    print_processes(&args, &processes);
}
