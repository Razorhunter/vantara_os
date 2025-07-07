use core::arch::asm;

pub const SYS_MOUNT: usize = 165;
pub const SYS_GETPID: usize = 39;
pub const SYS_MKDIR: usize = 83;
pub const SYS_MKNOD: usize = 133;

pub fn get_pid() -> isize {
    unsafe { syscall0(SYS_GETPID) }
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