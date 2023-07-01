#![no_std] 
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(naked_functions)]
#![feature(asm_sym)]

use core::panic::PanicInfo;
use bootloader::{BootInfo};
use x86_64::VirtAddr;
use memory::BootInfoFrameAllocator;
//use crate::threads::ThreadManager;
extern crate alloc;

#[macro_use]
mod vga;

mod cpu;
mod allocator;
mod memory;
mod threads;
mod arch;
mod gdt;

// Load in the root user space program
// include!("../elf_data.rs");

#[no_mangle]
pub extern "C" fn _start(boot_info: &'static BootInfo) -> ! {
    println!("Creating Interrupt Descriptor Table");
    gdt::init();
    cpu::init_idt();
    unsafe { cpu::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut kernel_table_mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };
    allocator::init_heap(&mut kernel_table_mapper, &mut frame_allocator).expect("heap initialization failed");

    //let mut thread_manager = ThreadManager::new(boot_info);
    //root_thread_init_memory(boot_info, &mut thread_manager);
    
    //println!("Root thread stack pointer: {:?}", thread_manager.get_stack_pointer(1));
    //println!("Root thread instruction pointer: {:?}", thread_manager.get_instruction_pointer(1));
 
    println!("Starting root thread");
    //thread_manager.switch_to(1);
    threads::new_kernel_thread(kernel_thread_main);
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
        cpu::hlt_loop();
    }
}

fn test_kernel_fn2() {
    println!("<< 2 >>");
    cpu::hlt_loop();
}

/*
fn root_thread_init_memory(boot_info: &'static BootInfo, thread_manager: &mut ThreadManager) {
    use bootloader::bootinfo::MemoryRegionType;
    use elf::endian::AnyEndian;
    use elf::abi::PT_LOAD;
    use elf::ElfBytes;

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut kernel_table_mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    // TODO: This should probably be moved to _start, I just didn't want to deal with the borrow checker for the mapper and frame_allocator
    allocator::init_heap(&mut kernel_table_mapper, &mut frame_allocator).expect("heap initialization failed");

    // Information about ELF structure can be found here https://en.wikipedia.org/wiki/Executable_and_Linkable_Format#Program_header
    // create page table entries while also loading the elf binary into memory
    let unused_regions = boot_info.memory_map.iter().filter(|r| r.region_type == MemoryRegionType::Usable);
    let mut region_count = 0;
    for region in unused_regions {
        println!("Unused Region: {:?}", region);
        let start_frame = PhysFrame::containing_address(PhysAddr::new(region.range.start_addr()));
        let end_frame = PhysFrame::containing_address(PhysAddr::new(region.range.end_addr()));

        // Add mappings to the kernel's page table so we can reference them to create the thread's page table
        println!("Updating kernel page table");
        for frame in PhysFrame::range_inclusive(start_frame, end_frame) {
            //println!("Frame: {:?}", frame);
            if frame.start_address().as_u64() >= 0x8000000 {
                // TODO: This is a hack to avoid mapping the heap, but it should be fixed
                continue;
            }
            let kernel_page = Page::containing_address(VirtAddr::new(frame.start_address().as_u64()));
            //println!("Kernel Page: {:?}", kernel_page);

            if region_count == 0 {
                memory::remove_mapping(kernel_page, &mut kernel_table_mapper);
                //println!("Original Kernel Mapping Removed");
            }

            memory::create_mapping(kernel_page, frame, &mut kernel_table_mapper, &mut frame_allocator);
            //println!("Kernel Mapping Created");
        }

        region_count += 1;
    }

    // parse elf binary contents in elf_data.rs, load into memory
    let file = ElfBytes::<AnyEndian>::minimal_parse(ELF_DATA).unwrap();
    
    for segment in file.segments().unwrap().iter() {
        if segment.p_type != PT_LOAD {
            continue;
        }
        let destination = segment.p_vaddr as usize;
        let source = &ELF_DATA[segment.p_offset as usize..][..segment.p_filesz as usize];
        crate::arch::elf::copy_memory(destination, source);
    }

    let entry_point: u64 = file.ehdr.e_entry;
    thread_manager.set_stack_pointer(1, 0x2000000);
    thread_manager.set_instruction_pointer(1, entry_point);
}
*/

/*
fn print_memory_map(boot_info: &'static BootInfo) {
    println!("Memory Map:");
    for region in boot_info.memory_map.iter() {
        let start_addr = region.range.start_addr();
        let end_addr = region.range.end_addr();
        println!("    [{:#016X}-{:#016X}] {:?}", start_addr, end_addr - 1, region.region_type);
    }
}
*/

/*
fn test_memory(boot_info: &'static BootInfo) {
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
*/