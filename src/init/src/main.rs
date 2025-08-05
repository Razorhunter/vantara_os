use std::fs::{create_dir_all, File};
use std::sync::Arc;
use std::ffi::CString;
use std::ptr;
use std::process::Command;
use libc;
use vantara::{safe_println, safe_eprintln, show_boot_banner};
use vantara::systemd::manager::ServiceManager;
use vantara::common::clear_screen;

fn main() {
    clear_screen();
    safe_println(format_args!("[BOOT] INIT Start"));

    create_directories_and_dev_nodes();
    mount_all_filesystems();
    clear_screen();
    vantara::setup::setup_firstboot();
    load_enable_services();
    clear_screen();
    show_boot_banner();

    if let Err(_e) = spawn_gui() {
        spawn_cli();
    }

    loop {
        ServiceManager::reap_children(); // collect all zombie process
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

fn create_directories_and_dev_nodes() {
    let dirs = [
        "/dev", "/dev/pts", "/proc", "/sys", "/mnt",
        "/run", "/usr", "/etc/service/available", "/etc/service/enabled"
    ];

    for dir in dirs {
        let _ = create_dir_all(dir);
    }

    let _ = File::create("/dev/console");
    let _ = File::create("/dev/null");
    let _ = File::create("/dev/tty");
}

fn mount_all_filesystems() {
    mount_fs("proc", "/proc", None);
    mount_fs("sysfs", "/sys", None);
    mount_fs("devtmpfs", "/dev", None);
    mount_fs("devpts", "/dev/pts", None);
    mount_fs("ext4", "/mnt", Some("/dev/sda"));
}

fn mount_fs(fstype: &str, target: &str, source_opt: Option<&str>) {
    let source = CString::new(source_opt.unwrap_or(fstype)).unwrap();
    let target = CString::new(target).unwrap();
    let fstype = CString::new(fstype).unwrap();

    let result = unsafe {
        libc::mount(
            source.as_ptr(),
            target.as_ptr(),
            fstype.as_ptr(),
            0,
            ptr::null(),
        )
    };

    if result != 0 {
        safe_eprintln(format_args!(
            "[ERR] Failed to mount {} on {}: {}",
            fstype.to_str().unwrap(),
            target.to_str().unwrap(),
            std::io::Error::last_os_error()
        ));
    } else {
        println!("[OK] Mounted {} on {}", fstype.to_str().unwrap(), target.to_str().unwrap());
    }
}

fn load_enable_services() {
    let manager = ServiceManager::new();
    ServiceManager::load_services(Arc::clone(&manager));

    {
        let mut m = manager.lock().unwrap();
        m.start_enabled_services();
    }
}

fn spawn_cli() {
    let path = CString::new("/bin/login").unwrap();
    let arg0 = CString::new("login").unwrap();
    let args = vec![arg0.as_ptr(), ptr::null()];
    let envp = vec![ptr::null()];

    unsafe {
        let pid = libc::fork();
        if pid == 0 {
            libc::setsid(); // Buat session baru (jadi pemilik terminal)
            libc::execve(path.as_ptr(), args.as_ptr(), envp.as_ptr());
            libc::_exit(1); // hanya dipanggil kalau execve gagal
        } else if pid > 0 {
            libc::waitpid(pid, ptr::null_mut(), 0); // Tunggu login tamat
        } else {
            safe_eprintln(format_args!("[ERR] Failed to fork login process."));
        }
    }
}

fn spawn_gui() -> Result<(), String> {
    match Command::new("/usr/bin/vantara_de")
        .spawn()
        .and_then(|mut child| child.wait())
    {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(format!("Keluar dengan kod: {}", status.code().unwrap_or(-1))),
        Err(e) => Err(format!("Gagal spawn DE: {e}")),
    }
}
