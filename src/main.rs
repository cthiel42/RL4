#![no_std] 
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use bootloader::{BootInfo};
use x86_64::{structures::paging::{Page, PageTable, PhysFrame, OffsetPageTable, Size4KiB, PageSize, Translate, RecursivePageTable}, VirtAddr, PhysAddr};
use memory::BootInfoFrameAllocator;
use crate::threads::ThreadManager;

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
    let mut thread_manager = ThreadManager::new(boot_info);
    root_thread_init_memory(boot_info, &mut thread_manager);
    
    println!("Root thread stack pointer: {:?}", thread_manager.get_stack_pointer(1));
    println!("Root thread instruction pointer: {:?}", thread_manager.get_instruction_pointer(1));

    println!("Starting root thread");
    thread_manager.switch_to(1);
    println!("Hello World");
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

fn root_thread_init_memory(boot_info: &'static BootInfo, thread_manager: &mut ThreadManager) {
    use bootloader::bootinfo::MemoryRegionType;
    use elf::endian::AnyEndian;
    use elf::abi::PT_LOAD;
    use elf::ElfBytes;
    use elf::segment::ProgramHeader;

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut kernel_table_mapper = unsafe { memory::init(phys_mem_offset) };
    let (mapper, frame_allocator) = thread_manager.get_page_table(1);

    // parse elf binary contents in elf_data.rs
    let file = ElfBytes::<AnyEndian>::minimal_parse(ELF_DATA).unwrap();
    let common_sections = file.find_common_data().unwrap();
    let first_load_phdr: Option<ProgramHeader> = file.segments().unwrap()
       .iter()
       .find(|phdr|{phdr.p_type == PT_LOAD});

    // create entrypoint
    let entry_point: u64 = first_load_phdr.unwrap().p_vaddr;
    let entry_page: u64 = entry_point / Size4KiB::SIZE as u64;
    println!("Entry point is at: {}", entry_point);
    println!("Entry page is at: {}", entry_page);
    println!("Len of ELF Data is: {}", ELF_DATA.len());

    // Information about ELF structure can be found here https://en.wikipedia.org/wiki/Executable_and_Linkable_Format#Program_header
    // create page table entries while also loading the elf binary into memory
    let unused_regions = boot_info.memory_map.iter().filter(|r| r.region_type == MemoryRegionType::Usable);
    let mut page_counter = 0;
    let mut elf_data_counter = 0;
    let mut region_count = 0;
    for region in unused_regions {
        println!("Unused Region: {:?}", region);
        let start_frame = PhysFrame::containing_address(PhysAddr::new(region.range.start_addr()));
        let end_frame = PhysFrame::containing_address(PhysAddr::new(region.range.end_addr()));

        // Add mappings to the kernel's page table so we can reference them to create the thread's page table
        println!("Updating kernel page table");
        for frame in PhysFrame::range_inclusive(start_frame, end_frame) {
            //println!("Frame: {:?}", frame);
            let kernel_page = Page::containing_address(VirtAddr::new(frame.start_address().as_u64()));
            //println!("Kernel Page: {:?}", kernel_page);

            if region_count == 0 {
                memory::remove_mapping(kernel_page, &mut kernel_table_mapper);
            }

            memory::create_mapping(kernel_page, frame, &mut kernel_table_mapper, frame_allocator);
            //println!("Kernel Mapping Created");

        }

        // Add mappings to the thread's page table
        println!("Creating thread page table");
        for frame in PhysFrame::range_inclusive(start_frame, end_frame) {
            if frame.start_address().as_u64() >= 0x1000000 {
                // println!("Frame: {:?}", frame);
                let page = Page::containing_address(VirtAddr::new(page_counter));
                // println!("Page: {:?}", page);
                memory::create_mapping(page, frame, mapper, frame_allocator);
                // println!("Thread Mapping created");

                // copy elf bytes into frame
                let frame_page = Page::containing_address(VirtAddr::new(frame.start_address().as_u64()));
                let mut frame_ptr: *mut u8 = frame_page.start_address().as_mut_ptr();
                
                // This is purely to let the compiler deduce the type of frame_page
                if 0 == 1 {
                    memory::create_mapping(frame_page, frame, &mut kernel_table_mapper, frame_allocator);
                }

                for _ in 0..512 {
                    if elf_data_counter >= ELF_DATA.len() {
                        break;
                    }
                    unsafe { frame_ptr.write_volatile(ELF_DATA[elf_data_counter]) };
                    unsafe { frame_ptr = frame_ptr.offset(1) };
                    elf_data_counter += 1;
                }
                page_counter += 4096;
            }
        }

        region_count += 1;
    }

    thread_manager.set_stack_pointer(1, 0x5000000);
    thread_manager.set_instruction_pointer(1, entry_point);

}

fn print_memory_map(boot_info: &'static BootInfo) {
    println!("Memory Map:");
    for region in boot_info.memory_map.iter() {
        let start_addr = region.range.start_addr();
        let end_addr = region.range.end_addr();
        println!("    [{:#016X}-{:#016X}] {:?}", start_addr, end_addr - 1, region.region_type);
    }
}

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