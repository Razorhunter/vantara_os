mod setup;

use std::fs::{create_dir_all, File};
use std::io::Write;
use std::process::{Command, Stdio};
use vantara::{safe_println, show_boot_banner};
use systemd::manager::ServiceManager;
use std::sync::Arc;

fn main() {
    clear_screen();
    safe_println(format_args!("[BOOT] INIT Start"));
    create_dev_node();
    mount_ext4();
    clear_screen();
    setup::setup_firstboot();
    load_enable_services();
    show_boot_banner();
    spawn_app();
}

fn load_enable_services() {
    let manager = ServiceManager::new();

    ServiceManager::load_services(Arc::clone(&manager));

    {
        let mut m = manager.lock().unwrap();
        m.start_enabled_services(); // still boleh guna macam ni
    }
}

fn mount_ext4() {
    safe_println(format_args!("[INFO] Mounting /dev/sda to /mnt as ext4"));
    let _ = Command::new("mount")
        .arg("-t")
        .arg("ext4")
        .arg("/dev/sda")
        .arg("/mnt")
        .status();
}

fn create_dev_node() {
    let _ = create_dir_all("/dev");
    let _ = create_dir_all("/mnt");
    let _ = create_dir_all("/run");
    let _ = create_dir_all("/usr");
    let _ = create_dir_all("/etc/service/available");
    let _ = create_dir_all("/etc/service/enabled");

    let _ = File::create("/dev/sda");
}

fn spawn_app() {
    let _ = Command::new("/bin/login")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();
}

fn clear_screen() {
    safe_println(format_args!("\x1B[2J\x1B[1;1H"));
    let _ = std::io::stdout().flush();
}
