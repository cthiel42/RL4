#![no_std] 
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(naked_functions)]

use core::panic::PanicInfo;
use bootloader::{BootInfo};
extern crate alloc;

#[macro_use]
mod vga;

mod cpu;
mod allocator;
mod memory;
mod threads;
mod arch;
mod gdt;

#[no_mangle]
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    println!("Creating Interrupt Descriptor Table");
    gdt::init();
    cpu::init_idt();
    unsafe { memory::init(boot_info) };
    unsafe { cpu::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

    println!("Starting root thread");
    // threads::new_kernel_thread(kernel_thread_main);
    let _ = threads::new_user_thread(include_bytes!("../user_space/hello_world/target/target/debug/hello_world"));
    println!("Hello World from the kernel!");
    cpu::hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    cpu::hlt_loop();
}

fn kernel_thread_main() {
    println!("Kernel thread start");
    threads::new_kernel_thread(test_kernel_fn2);
    loop {
        println!("<< 1 >>");
        x86_64::instructions::hlt();
    }
}

fn test_kernel_fn2() {
    loop {
        println!("<< 2 >>");
        x86_64::instructions::hlt();
    }
}