use core::arch::asm;

pub const SYS_MOUNT: usize = 165;
pub const SYS_GETPID: usize = 39;
pub const SYS_MKDIR: usize = 83;
pub const SYS_MKNOD: usize = 133;
pub const SYS_FORK: usize = 57;
pub const SYS_EXECVE: usize = 59;
pub const SYS_EXIT: usize = 60;
pub const SYS_WAITPID: usize = 61;
pub const SYS_OPEN: usize = 2;
pub const SYS_DUP2: usize = 33;
pub const SYS_CLOSE: usize = 6;
pub const SYS_READ: usize = 0;
pub const SYS_WRITE: usize = 1;

pub fn get_pid() -> isize {
    unsafe { syscall0(SYS_GETPID) }
}

pub fn write(msg: &[u8]) {
    unsafe {
        syscall3(1, 1, msg.as_ptr() as usize, msg.len()); // write stdout
    }
}

pub fn write_num(mut num: usize) {
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

pub unsafe fn syscall0(n: usize) -> isize {
    let ret: isize;
    core::arch::asm!(
        "syscall",
        in("rax") n,
        lateout("rax") ret,
        lateout("rcx") _,
        lateout("r11") _,
    );
    ret
}

pub unsafe fn syscall1(n: usize, a1: usize) -> isize {
    let ret: isize;
    asm!("syscall", in("rax") n, in("rdi") a1, lateout("rax") ret, lateout("rcx") _, lateout("r11") _);
    ret
}

pub unsafe fn syscall2(n: usize, a1: usize, a2: usize) -> isize {
    let ret: isize;
    asm!("syscall", in("rax") n, in("rdi") a1, in("rsi") a2, lateout("rax") ret, lateout("rcx") _, lateout("r11") _);
    ret
}

pub unsafe fn syscall3(n: usize, a1: usize, a2: usize, a3: usize) -> isize {
    let ret: isize;
    asm!("syscall", in("rax") n, in("rdi") a1, in("rsi") a2, in("rdx") a3, lateout("rax") ret, lateout("rcx") _, lateout("r11") _);
    ret
}

pub unsafe fn syscall4(n: usize, a1: usize, a2: usize, a3: usize, a4: usize) -> isize {
    let ret: isize;
    asm!("syscall", in("rax") n, in("rdi") a1, in("rsi") a2, in("rdx") a3, in("r10") a4, lateout("rax") ret, lateout("rcx") _, lateout("r11") _);
    ret
}

pub unsafe fn syscall5(n: usize, a1: usize, a2: usize, a3: usize, a4: usize, a5: usize) -> isize {
    let ret: isize;
    asm!("syscall", in("rax") n, in("rdi") a1, in("rsi") a2, in("rdx") a3, in("r10") a4, in("r8") a5, lateout("rax") ret, lateout("rcx") _, lateout("r11") _);
    ret
}