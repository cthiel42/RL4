#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::arch::asm;

#[no_mangle]
pub unsafe extern "sysv64" fn _start() {
    loop {
        let s = "<< 1 >>";
        unsafe {
            asm!("mov rax, 1", // write syscall function
                "syscall",
                in("rdi") s.as_ptr(), // First argument
                in("rsi") s.len()); // Second argument
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}