#![no_std] 
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use bootloader::{BootInfo};
use x86_64::{structures::paging::{Page, PhysFrame, OffsetPageTable}, VirtAddr, PhysAddr};
use memory::BootInfoFrameAllocator;

#[macro_use]
mod vga;

mod cpu;
mod memory;
mod threads;
mod arch;

// Load in the root user space program
include!("../elf_data.rs");

#[no_mangle]
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    println!("Creating Interrupt Descriptor Table");
    cpu::init_idt();
    println!("Initializing root thread memory");
    root_thread_init_memory(boot_info);
    println!("Hello World");
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

fn root_thread_init_memory(boot_info: &'static BootInfo) -> (OffsetPageTable, memory::BootInfoFrameAllocator, VirtAddr) {
    use bootloader::bootinfo::MemoryRegionType;
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    // find all unused memory regions
    let unused_regions = boot_info.memory_map.iter().filter(|r| r.region_type == MemoryRegionType::Usable);

    // map each unused region to a page in the new page table and keep track of highest address for stack pointer
    let mut page_counter = 0;
    for region in unused_regions {
        println!("Unused Region: {:?}", region);
        let start_frame = PhysFrame::containing_address(PhysAddr::new(region.range.start_addr()));
        let end_frame = PhysFrame::containing_address(PhysAddr::new(region.range.end_addr()));
        for frame in PhysFrame::range_inclusive(start_frame, end_frame) {
            // println!("Frame: {:?}", frame);
            let page = Page::containing_address(VirtAddr::new(page_counter));
            page_counter += 4096;
            // println!("Page: {:?}", page);
            memory::create_mapping(page, frame, &mut mapper, &mut frame_allocator);
        }
    }

    // set stack pointer to highest address in page table
    let stack_pointer = VirtAddr::new(page_counter-4096);

    // TODO: load root binary or ELF into memory
    // retrieve elf binary from contents of elf_data.rs


    // TODO: assign instruction pointer? Possibly from ELF header

    (mapper, frame_allocator, stack_pointer)
}

fn test_memory(boot_info: &'static BootInfo) {
    use x86_64::{structures::paging::Translate};
    // Testing virtual memory addresses
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mapper = unsafe { memory::init(phys_mem_offset) };

    let addresses = [
        // vga buffer page
        0xb8000,
        // code page
        0x201008,
        // stack page
        0x0100_0020_1a10,
        // virtual address mapped to physical address 0
        boot_info.physical_memory_offset,
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = mapper.translate_addr(virt);
        println!("{:?} -> {:?}", virt, phys);
    }    

    // Testing for new page table creation 
    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    // map an unused page
    let page = Page::containing_address(VirtAddr::new(0));
    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    memory::create_mapping(page, frame, &mut mapper, &mut frame_allocator);

    // write the string `New!` to the screen through the new mapping
    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e)};

    // map a page that doesn't exist to the vga buffer
    let page = Page::containing_address(VirtAddr::new(0xdeadbeaf000));
    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    memory::create_mapping(page, frame, &mut mapper, &mut frame_allocator);
}