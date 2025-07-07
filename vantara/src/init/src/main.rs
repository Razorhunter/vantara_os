#![no_std]
#![no_main]

mod syscall;
mod service;
use core::ptr;
use core::panic::PanicInfo;
use crate::syscall::{syscall1, syscall2, syscall3, syscall4, syscall5, get_pid, SYS_FORK, SYS_WAITPID, SYS_MOUNT, SYS_MKDIR, SYS_MKNOD, write};
use crate::service::{SERVICES, start_service, RestartPolicy, can_start, service_control_loop};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    write(b"\x1B[2J\x1B[1;1H"); // Clear screen
    let pid = get_pid();

    if pid == 1 {
        write(b"[BOOT] INIT Start\n");
        create_dev_node();
        mount_ext4();

        unsafe {
            let mut started = 0;
            let total = SERVICES.len();

            while started < total {
                for s in SERVICES.iter_mut() {
                    if s.pid.is_none() && can_start(s) {
                        write(b"[INIT] Starting service: ");
                        write(s.name);
                        write(b"\n");
                        start_service(s);
                        started += 1;
                    }
                }
            }

            let ctl_pid = syscall1(SYS_FORK, 0);
            if ctl_pid == 0 {
                service_control_loop(); //service control loop disini
            }
        }

        write(b"\x1B[2J\x1B[1;1H"); // Clear screen
        spawn_login();
    }

    loop {
        unsafe {
            let mut status: i32 = 0;
            let exited_pid = syscall4(SYS_WAITPID, -1isize as usize, &mut status as *mut i32 as usize, 0, 0) as usize;

            if exited_pid < 0 {
                continue;
            }

            let exit_code = ((status >> 8) & 0xFF) as u8;

            for s in SERVICES.iter_mut() {
                if Some(exited_pid) == s.pid {
                    write(b"[INFO] Service exited: ");
                    write(s.name);
                    write(b"\n");

                    s.pid = None; //Reset the pid

                    match s.restart {
                        RestartPolicy::Always => {
                            write(b"[RESTART] Always policy...\n");
                            start_service(s);
                        },
                        RestartPolicy::OnFailure => {
                            if exit_code != 0 {
                                write(b"[RESTART] Failed, restarting...\n");
                                start_service(s);
                            } else {
                                write(b"[RESTART] Clean exit, not restarting\n")
                            }
                        },
                        RestartPolicy::Never => {
                            write(b"[RESTART] Never policy, not restarting.\n");
                        }
                    }
                }
            }
        }
    }
}

fn mount_ext4() {
    let source = b"/dev/sda\0";
    let target = b"/mnt\0";
    let fstype = b"ext4\0";

    unsafe {
        let _res = syscall5(
            SYS_MOUNT,
            source.as_ptr() as usize,
            target.as_ptr() as usize,
            fstype.as_ptr() as usize,
            0,
            0,
        );
    }
}

fn create_dev_node() {
    let dev = b"/dev\0";
    let mnt = b"/mnt\0";
    let run = b"/run\0";
    let pipe = b"/run/servicectl\0";
    let pipe_mode = 0o644 | (1 << 12);
    let fifo_type = 0o10000;

    mkdir(dev, 0o755);
    mkdir(mnt, 0o755);
    mkdir(run, 0o755);

    let sda1 = b"/dev/sda\0";
    let major = 8;
    let minor = 1;
    let devnum = (major << 8) | minor;

    unsafe {
        syscall4(SYS_MKNOD, sda1.as_ptr() as usize, 0o600, devnum, 0);
        syscall4(SYS_MKNOD, pipe.as_ptr() as usize, fifo_type | pipe_mode, 0, 0);
    }
}

fn spawn_login() {
    let path = b"/bin/login\0";
    let arg0 = b"login\0";
    let argv = [arg0.as_ptr(), ptr::null()];

    unsafe {
        let pid = syscall1(57, 0); // fork
        if pid == 0 {
            syscall3(59, path.as_ptr() as usize, argv.as_ptr() as usize, 0); // execve
            syscall1(60, 1); // exit if failed
        } else {
            syscall1(61, pid as usize); // wait
        }
    }
}

fn mkdir(path: &[u8], mode: usize) {
    unsafe {
        syscall2(SYS_MKDIR, path.as_ptr() as usize, mode);
    }
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub unsafe extern "C" fn memcpy(dest: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *dest.add(i) = *src.add(i);
        i += 1;
    }
    dest
}

#[no_mangle]
pub unsafe extern "C" fn memset(s: *mut u8, c: i32, n: usize) -> *mut u8 {
    let mut i = 0;
    while i < n {
        *s.add(i) = c as u8;
        i += 1;
    }
    s
}

#[no_mangle]
pub unsafe extern "C" fn memcmp(s1: *const u8, s2: *const u8, n: usize) -> i32 {
    let mut i = 0;
    while i < n {
        let a = *s1.add(i);
        let b = *s2.add(i);
        if a != b {
            return (a as i32) - (b as i32);
        }
        i += 1;
    }
    0
}
