use core::ptr;
use core::arch::asm;
use crate::syscall::{syscall1, syscall2, syscall3, syscall4, write, write_num, SYS_CLOSE, SYS_READ, SYS_MKNOD, SYS_FORK, SYS_EXECVE, SYS_EXIT, SYS_DUP2, SYS_OPEN};

pub enum RestartPolicy {
    Never,
    Always,
    OnFailure,
}

pub struct Service<'a> {
    pub name: &'a [u8],
    path: &'a [u8],
    args: &'a [&'a [u8]],
    pub pid: Option<usize>,
    pub restart: RestartPolicy,
    start_after: &'a [&'a [u8]],
    pub enabled: bool,
}

pub static mut SERVICES: [Service; 1] = [
    Service {
        name: b"sshd\0",
        path: b"/bin/sshd\0",
        args: &[],
        pid: None,
        restart: RestartPolicy::Never,
        start_after: &[],
        enabled: true
    }
];

pub unsafe fn service_control_loop() {
    let path = b"/run/servicectl\0";
    let fd = syscall3(SYS_OPEN, path.as_ptr() as usize, 0x242, 0o644); // O_CREAT | O_RDWR | O_TRUNC

    if fd < 0 {
        write(b"[ERROR] Cannot open control file\n");
        return;
    }

    let mut buf = [0u8; 64];

    loop {
        let n = syscall3(SYS_READ, fd as usize, buf.as_mut_ptr() as usize, 63); // baca maksimum 63 byte

        if n > 0 {
            buf[n as usize] = 0; // null-terminate
            handle_command(&buf);
        } else if n == 0 {
            // pipe kosong, tak buat apa-apa
        } else {
            write(b"[ERROR] Failed reading from control file\n");
            break; // keluar dari loop kalau read gagal
        }

        // Delay supaya CPU tak 100%
        for _ in 0..5000000 {
            core::arch::asm!("nop");
        }
    }

    // Tutup file descriptor bila keluar loop (optional)
    let _ = syscall1(SYS_CLOSE, fd as usize);
}

unsafe fn handle_command(cmd: &[u8]) {
    let mut tmp = [0u8; 16];
    if cmd.starts_with(b"start ") {
        copy_name(&cmd[6..], &mut tmp);
        for s in SERVICES.iter_mut() {
            if strcmp(s.name, &tmp) {
                start_service(s);
                return;
            }
        }
        write(b"[ERROR] Service not found\n");
    } else if cmd.starts_with(b"stop ") {
        copy_name(&cmd[5..], &mut tmp);
        stop_service(&tmp);
    } else if cmd.starts_with(b"restart ") {
        copy_name(&cmd[8..], &mut tmp);
        restart_service(&tmp);
    } else if cmd.starts_with(b"status ") {
        copy_name(&cmd[7..], &mut tmp);
        status_service(&tmp);
    } else if cmd.starts_with(b"enable ") {
        copy_name(&cmd[7..], &mut tmp);
        enable_service(&tmp);
    } else if cmd.starts_with(b"disable ") {
        copy_name(&cmd[8..], &mut tmp);
        enable_service(&tmp);
    } else if cmd.starts_with(b"list") {
        list_service();
    } else {
        write(b"[ERROR] Unknown command\n");
    }
}

pub unsafe fn start_service(service: &mut Service) {
    let pid = syscall1(SYS_FORK, 0);
    if pid == 0 {
        let argv = [service.args[0].as_ptr(), ptr::null()];
        // redirect_stdio(service.name);
        syscall3(SYS_EXECVE, service.path.as_ptr() as usize, argv.as_ptr() as usize, 0);
        syscall1(SYS_EXIT, 1);
    } else {
        service.pid = Some(pid as usize);
        write(b"[INFO] Started service: ");
        write(service.name);
        write(b" (pid: ");
        write_num(service.pid.unwrap_or(0));
        write(b")");
        write(b"\n");
    }
}

pub unsafe fn status_service(name: &[u8]) {
    for s in SERVICES.iter() {
        if strcmp(s.name, name) {
            write(b"[STATUS] ");
            write(s.name);
            write(b": ");

            match s.pid {
                Some(pid) => {
                    write(b"RUNNING (pid: ");
                    write_num(pid);
                    write(b")\n");
                },
                None => {
                    write(b"STOPPED\n");
                }
            }
            return;
        }
    }

    write(b"[ERROR] Service not found\n");
}

pub unsafe fn stop_service(name: &[u8]) {
    for s in SERVICES.iter_mut() {
        if strcmp(s.name, name) {
            match s.pid {
                Some(pid) => {
                    syscall2(62, pid, 15); // kill(pid, SIGTERM)
                    s.pid = None;

                    write(b"[STOP] ");
                    write(s.name);
                    write(b"\n");
                }
                None => {
                    write(b"[INFO] Already stopped: ");
                    write(s.name);
                    write(b"\n");
                }
            }
            return;
        }
    }

    write(b"[ERROR] Service not found\n");
}

pub unsafe fn restart_service(name: &[u8]) {
    stop_service(name);

    for s in SERVICES.iter_mut() {
        if strcmp(s.name, name) {
            start_service(s);
            return;
        }
    }
}

pub unsafe fn list_service() {
    for s in SERVICES.iter_mut() {
        write(b"- ");
        write(s.name);
        if let Some(pid) = s.pid {
            write(b": Running (PID ");
            write_num(pid);
            write(b")\n");
        } else {
            write(b": Stopped\n");
        }
    }
}

pub unsafe fn enable_service(name: &[u8]) {
    for s in SERVICES.iter_mut() {
        if s.name == name {
            s.enabled = true;
            write(b"[SERVCTL] Enabled: ");
            write(name);
            write(b"\n");
            return;
        }
    }
}

pub unsafe fn disable_service(name: &[u8]) {
    for s in SERVICES.iter_mut() {
        if s.name == name {
            s.enabled = false;
            write(b"[SERVCTL] Enabled: ");
            write(name);
            write(b"\n");
            return;
        }
    }
}

unsafe fn is_service_started(name: &[u8]) -> bool {
    for s in SERVICES.iter() {
        if strcmp(s.name, name) && s.pid.is_some() {
            return true;
        }
    }
    false
}

pub unsafe fn can_start(service: &Service) -> bool {
    for dep in service.start_after {
        if !is_service_started(dep) {
            return false;
        }
    }
    true
}

fn strcmp(a: &[u8], b: &[u8]) -> bool {
    let mut i = 0;
    while i < a.len() && i < b.len() {
        if a[i] != b[i] {
            return false;
        }
        if a[i] == 0 {
            return true; // dua-dua sama dan berakhir dengan NULL
        }
        i += 1;
    }

    // Cegah false positive bila satu string habis awal
    if i < a.len() && a[i] != 0 {
        return false;
    }
    if i < b.len() && b[i] != 0 {
        return false;
    }

    true
}

unsafe fn redirect_stdio(name: &[u8]) {
    let mut log_path = [0u8; 64];
    let prefix = b"/run/";
    let suffix = b".log\0";

    let mut i = 0;
    for &b in prefix {
        log_path[i] = b;
        i += 1;
    }
    for &b in name {
        if b == 0 { break; }
        log_path[i] = b;
        i += 1;
    }
    for &b in suffix {
        log_path[i] = b;
        i += 1;
    }

    // O_CREAT | O_WRONLY | O_TRUNC = 0o1 | 0o100 | 0o1000 = 0x241
    let fd = syscall3(SYS_OPEN, log_path.as_ptr() as usize, 0x241, 0o644);

    if fd >= 0 {
        syscall2(SYS_DUP2, fd as usize, 1); // stdout
        syscall2(SYS_DUP2, fd as usize, 2); // stderr
    } else {
        // fallback: buka /dev/tty
        let tty = b"/dev/tty\0";
        let tty_fd = syscall3(SYS_OPEN, tty.as_ptr() as usize, 0x1, 0); // O_WRONLY

        if tty_fd >= 0 {
            syscall2(SYS_DUP2, tty_fd as usize, 1); // stdout
            syscall2(SYS_DUP2, tty_fd as usize, 2); // stderr
        }

        // optional: tulis error ke original stdout
        write(b"[WARN] log redirect failed, fallback to /dev/tty\n");
    }
}

fn copy_name(src: &[u8], dst: &mut [u8]) {
    let mut i = 0;
    while i < src.len() && i < dst.len() - 1 {
        if src[i] == b'\n' { break; }
        dst[i] = src[i];
        i += 1;
    }
    dst[i] = 0;
}

