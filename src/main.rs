#![no_std] 
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use core::arch::asm;
use bootloader::{BootInfo};
use x86_64::{structures::paging::{Page, PageTable, PhysFrame, OffsetPageTable, Size4KiB, PageSize, Translate, RecursivePageTable}, VirtAddr, PhysAddr};
use x86_64::registers::control::{Cr3,Cr3Flags};
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
    let (stack_pointer, entry_point) = root_thread_init_memory(boot_info);
    println!("Starting root thread");
    //root_thread_start(page_table, stack_pointer, entry_point);
    println!("Hello World");
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

fn root_thread_init_memory(boot_info: &'static BootInfo) -> (VirtAddr, VirtAddr) {
    use bootloader::bootinfo::MemoryRegionType;
    use elf::endian::AnyEndian;
    use elf::abi::PT_LOAD;
    use elf::ElfBytes;
    use elf::segment::ProgramHeader;

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut kernel_table_mapper = unsafe { memory::init(phys_mem_offset) };
    let mut page_table = memory::new_page_table();
    let mut mapper = memory::new_mapper(&mut page_table);
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    // parse elf binary contents in elf_data.rs
    let file = ElfBytes::<AnyEndian>::minimal_parse(ELF_DATA).unwrap(); // this line here does something that fucks up my memory. maybe delete it? work around it?
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

    // cr3 register testing
    println!("Page table test");
    // This code breaks everything below it from running and I haven't the slightest clue why
    // Using the x86_64 crate to do the same thing works fine, and I haven't the slightest clue why
    // let cr3: u64;
    // unsafe {
    //     asm!("mov cr3, {}", out(reg) cr3);
    //     asm!("mov {}, cr3", in(reg) cr3);
    // }
    let (mut kernel_page_table,_) = Cr3::read();
    let kernel_page_table_pointer: u64 = &kernel_page_table as *const _ as u64;
    unsafe { Cr3::write(kernel_page_table, Cr3Flags::empty()); }
    println!("CR3: {}", kernel_page_table_pointer);
    // end test

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
        for frame in PhysFrame::range_inclusive(start_frame, end_frame) {
            //println!("Frame: {:?}", frame);
            let kernel_page = Page::containing_address(VirtAddr::new(frame.start_address().as_u64()));
            //println!("Kernel Page: {:?}", kernel_page);

            // The first unused region is already mapped by the kernel, so we remove and recreate that
            // mapping in order to set the permissions correctly
            if region_count == 0 {
                memory::remove_mapping(kernel_page, &mut kernel_table_mapper);
                // println!("Kernel Mapping Removed");
            }

            memory::create_mapping(kernel_page, frame, &mut kernel_table_mapper, &mut frame_allocator);
            //println!("Kernel Mapping Created");
        }

        // Add mappings to the thread's page table
        for frame in PhysFrame::range_inclusive(start_frame, end_frame) {
            //println!("Frame: {:?}", frame);
            let page = Page::containing_address(VirtAddr::new(page_counter));
            //println!("Page: {:?}", page);
            memory::create_mapping(page, frame, &mut mapper, &mut frame_allocator);
            //println!("Thread Mapping created");

            page_counter += 4096;

            // copy elf bytes into frame
            /* let mut page_ptr: *mut u8 = page.start_address().as_mut_ptr();
            for _ in 0..512 {
                if elf_data_counter >= ELF_DATA.len() {
                    break;
                }
                unsafe { page_ptr.write_volatile(ELF_DATA[elf_data_counter]) };
                unsafe { page_ptr = page_ptr.offset(1) };
                elf_data_counter += 1;
            } */
        }

        // TODO: Remove mappings from kernel page table?

        region_count += 1;
    }

    let stack_pointer = VirtAddr::new(page_counter-4096);

    (stack_pointer, VirtAddr::new(entry_point))
}

fn root_thread_start(mut page_table: OffsetPageTable, stack_pointer: VirtAddr, entry_point: VirtAddr) {
    unsafe { 
        // Load the page table into the CR3 register
        let mut level_4_table = page_table.level_4_table();
        let level_4_table_pointer: u64 = level_4_table as *const _ as u64;
        println!("Level 4 Table Pointer: {:x}", level_4_table_pointer);
        Cr3::write(PhysFrame::containing_address(PhysAddr::new(level_4_table_pointer)), Cr3Flags::empty()); 

        // Set the stack pointer
        let stack_pointer = stack_pointer.as_u64();
        asm!("mov rsp, {}", in(reg) stack_pointer);

        // Jump to the entry point
        let entry_point = entry_point.as_u64();
        asm!("jmp {}", in(reg) entry_point);
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