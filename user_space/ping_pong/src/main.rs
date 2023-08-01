#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::arch::asm;

#[no_mangle]
pub unsafe extern "sysv64" fn _start() {
    let s = "hello world from ping pong";
    unsafe {
        asm!("mov rax, 1", // write syscall function
             "syscall",
             in("rdi") s.as_ptr(), // First argument
             in("rsi") s.len()); // Second argument
    }

    // receive ipc message
    let mut msg: u64 = 0;
    let mut err: u64 = 0;
    unsafe {
        asm!("mov rax, 3", // read ipc function
             "mov rdi, 0",
             "syscall",
             lateout("rax") err,
             lateout("rdi") msg); // First argument
    }

    let s = "hello world from ping pong again";
    unsafe {
        asm!("mov rax, 1", // write syscall function
             "syscall",
             in("rdi") s.as_ptr(), // First argument
             in("rsi") s.len()); // Second argument
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}