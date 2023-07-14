use core::arch::asm;
use core::{slice, str, ptr};

const MSR_STAR: usize = 0xc0000081;
const MSR_LSTAR: usize = 0xc0000082;
const MSR_FMASK: usize = 0xc0000084;

#[naked]
extern "C" fn handle_syscall() {
    unsafe {
        asm!(
            // Switch to kernel stack and backup registers for sysretq
            "push rcx",
            "push r11",
            "push rbp",
            "push rbx", // save callee registers
            "push r12",
            "push r13",
            "push r14",
            "push r15",

            "cmp rax, 0",       // if rax == 0 {
            "jne 1f",
            "call {hello_world}",  //   hello_world();
            "1: cmp rax, 1",    // } if rax == 1 {
            "jne 2f",
            "call {sys_write}", //   sys_write();
            "2: ",              // }

            "pop r15", // restore callee registers
            "pop r14",
            "pop r13",
            "pop r12",
            "pop rbx",
            "pop rbp", // restore stack and registers for sysretq
            "pop r11",
            "pop rcx",
            "sysretq", // back to userspace
            sys_write = sym sys_write,
            hello_world = sym hello_world,
            options(noreturn)
        );
    }
}

pub fn init() {
    let handler_addr = handle_syscall as *const () as u64;
    unsafe {
        asm!("mov ecx, 0xC0000080",
            "rdmsr",
            "or eax, 1",
            "wrmsr"
        );
        asm!("xor rdx, rdx",
            "mov rax, 0x200",
            "wrmsr",
            in("rcx") MSR_FMASK
        );
        asm!("mov rdx, rax",
            "shr rdx, 32",
            "wrmsr",
            in("rax") handler_addr,
            in("rcx") MSR_LSTAR
        );
        asm!(
            "xor rax, rax",
            "mov rdx, 0x230008",
            "wrmsr",
            in("rcx") MSR_STAR
        );
    }
}

extern "C" fn sys_write(ptr: *mut u8, len:usize) {
    let u8_slice = unsafe {slice::from_raw_parts(ptr, len)};

    if let Ok(s) = str::from_utf8(u8_slice) {
        println!("Write '{}'", s);
    } else {
        println!("Write failed");
    }
}

extern "C" fn hello_world() {
    println!("Hello World");
}


