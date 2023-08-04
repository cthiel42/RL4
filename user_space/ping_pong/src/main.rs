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

    let array_of_strings: [&str; 30] = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12", "13", "14", "15", "16", "17", "18", "19", "20", "21", "22", "23", "24", "25", "26", "27", "28", "29", "30"];
    for s in &array_of_strings{
        unsafe {
            asm!("mov rax, 1", // write syscall function
                "syscall",
                in("rdi") s.as_ptr(), // First argument
                in("rsi") s.len()); // Second argument
        }
        for _ in 1..10000000 {
            unsafe { asm!("nop"); }
        }

        // Yield
        unsafe {
            asm!("mov rax, 4",
                "syscall");
        }
    }

    loop {
        unsafe {
            asm!("mov rax, 4",
                "syscall");
        }
    }

    /*
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
    */
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}