#![no_std]
#![no_main]

mod syscall;

use core::ptr;
use core::arch::asm;
use core::panic::PanicInfo;

use crate::syscall::{syscall0, syscall1, syscall2, syscall3, syscall4, syscall5, get_pid, SYS_MOUNT, SYS_MKDIR, SYS_MKNOD};

const SYS_FORK: usize = 57;
const SYS_EXECVE: usize = 59;
const SYS_EXIT: usize = 60;
const SYS_WAITPID: usize = 61;

struct Service<'a> {
    name: &'a [u8],
    path: &'a [u8],
    args: &'a [&'a [u8]],
    pid: Option<usize>,
}

static mut SERVICES: [Service; 2] = [
    Service {
        name: b"login\0",
        path: b"/bin/login\0",
        args: &[b"login\0"],
        pid: None,
    },
    Service {
        name: b"sshd\0",
        path: b"/bin/sshd\0",
        args: &[b"sshd\0"],
        pid: None,
    }
];

#[no_mangle]
pub extern "C" fn _start() -> ! {
    write(b"\x1B[2J\x1B[1;1H"); // Clear screen)
    let pid = get_pid();

    if pid == 1 {
        create_dev_node();
        mount_ext4();

        unsafe {
            for s in SERVICES.iter_mut() {
                start_service(s);
            }
        }
    }

    loop {
        unsafe {
            let exited_pid = syscall1(SYS_WAITPID, -1isize as usize) as usize;
            for s in SERVICES.iter_mut() {
                if Some(exited_pid) == s.pid {
                    write(b"[INFO] Service exited: ");
                    write(s.name);
                    write(b"\n");

                    start_service(s); //restart killed service
                }
            }
        }
    }
}


fn start_service(service: &mut Service) {
    unsafe {
        let pid = syscall1(SYS_FORK, 0);
        if pid == 0 {
            let argv = [service.args[0].as_ptr(), ptr::null()];
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

    mkdir(dev, 0o755);
    mkdir(mnt, 0o755);

    let sda1 = b"/dev/sda\0";
    let major = 8;
    let minor = 1;
    let devnum = (major << 8) | minor;

    unsafe {
        syscall4(SYS_MKNOD, sda1.as_ptr() as usize, 0o600, devnum, 0);
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

fn write(msg: &[u8]) {
    unsafe {
        syscall3(1, 1, msg.as_ptr() as usize, msg.len()); // write stdout
    }
}

fn write_num(mut num: usize) {
    let mut buf = [0u8; 20]; // max 20 digit untuk 64-bit integer
    let mut i = buf.len();

    if num == 0 {
        write(b"0\n");
        return;
    }

    while num > 0 {
        i -= 1;
        buf[i] = b'0' + (num % 10) as u8;
        num /= 10;
    }

    write(&buf[i..]);
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
