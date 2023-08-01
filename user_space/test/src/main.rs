#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::arch::asm;

#[no_mangle]
pub unsafe extern "sysv64" fn _start() {
    let s = "Starting Ping Pong Thread Number 1";
    unsafe {
        asm!("mov rax, 1", // write syscall function
            "syscall",
            in("rdi") s.as_ptr(), // First argument
            in("rsi") s.len()); // Second argument
    }

    unsafe {
        asm!("mov rax, 4", // write syscall function
             "syscall");
    }

    let s = "Starting Ping Pong Thread Number 2";
    unsafe {
        asm!("mov rax, 1", // write syscall function
             "syscall",
             in("rdi") s.as_ptr(), // First argument
             in("rsi") s.len()); // Second argument
    }

    unsafe {
        asm!("mov rax, 4", // write syscall function
             "syscall");
    }

    let s = "Both Ping Pong Threads are running";
    unsafe {
        asm!("mov rax, 1", // write syscall function
             "syscall",
             in("rdi") s.as_ptr(), // First argument
             in("rsi") s.len()); // Second argument
    }

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}