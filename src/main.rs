#![no_std] 
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;

#[macro_use]
mod vga;

mod cpu;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Creating Interrupt Descriptor Table");
    cpu::init_idt();
    println!("Hello World");
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}