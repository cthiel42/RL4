#![no_std] 
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(naked_functions)]
#![feature(asm_const)]

use core::panic::PanicInfo;
use bootloader::{BootInfo};
use spin::RwLock;
use lazy_static::lazy_static;
extern crate alloc;
use alloc::vec::Vec;

#[macro_use]
mod vga;

mod cpu;
mod allocator;
mod memory;
mod threads;
mod arch;
mod gdt;
mod syscalls;
mod ipc;


lazy_static! {
    static ref RENDEZVOUS: alloc::sync::Arc<RwLock<ipc::Rendezvous>> = alloc::sync::Arc::new(RwLock::new(ipc::Rendezvous::Empty));
}

#[no_mangle]
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    println!("Creating Interrupt Descriptor Table");
    gdt::init();
    syscalls::init();
    cpu::init_idt();
    unsafe { memory::init(boot_info) };
    unsafe { cpu::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

    println!("Starting root thread");
    threads::new_kernel_thread(start_ping_pong);
    println!("Hello World from the kernel!");
    cpu::hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    cpu::hlt_loop();
}

fn start_ping_pong() {
    println!("kernel thread started");
    // threads::new_kernel_thread(kernel_thread_main);
    let thread1 = threads::new_user_thread(include_bytes!("../user_space/ping_pong/target/target/debug/ping_pong"), Vec::from([RENDEZVOUS.clone()]));
    let thread2 = threads::new_user_thread(include_bytes!("../user_space/ping_pong/target/target/debug/ping_pong"), Vec::from([RENDEZVOUS.clone()]));
    println!("Threads created. Adding them to the queue");
    threads::schedule_thread(thread1);
    threads::schedule_thread(thread2);
    println!("Ping pong threads created - kernel");
    loop {
        println!("<< 0 >>");
        x86_64::instructions::hlt();
    }
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