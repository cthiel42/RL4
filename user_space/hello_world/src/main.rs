#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
pub unsafe extern "sysv64" fn _start() {
    // write hello world to the vga buffer
    let vga_buffer = 0xb8000 as *mut u8;
    for (i, &byte) in b"Hello World!".iter().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte;
            *vga_buffer.offset(i as isize * 2 + 1) = 0xb;
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}