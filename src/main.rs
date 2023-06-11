#![no_std] 
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use bootloader::{BootInfo};
use x86_64::{structures::paging::{Page, PhysFrame, OffsetPageTable, Size4KiB, PageSize}, VirtAddr, PhysAddr};
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

fn root_thread_init_memory(boot_info: &'static BootInfo) -> (OffsetPageTable, memory::BootInfoFrameAllocator, VirtAddr, VirtAddr) {
    use bootloader::bootinfo::MemoryRegionType;
    use elf::endian::AnyEndian;
    use elf::abi::PT_LOAD;
    use elf::ElfBytes;
    use elf::segment::ProgramHeader;

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    // retrieve elf binary from contents of elf_data.rs
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

    // map the elf into a page table
    for program_header in file.segments().unwrap().iter() {
        let segment_virtual_address: u64 = program_header.p_vaddr; // virtual address the segment should be loaded
        let segment_physical_address: u64 = program_header.p_paddr; // physical address the segment should be loaded
        let segment_size: u64 = program_header.p_memsz; // size of the segment in bytes
        println!("Segment virtual address: {}", segment_virtual_address);
        println!("Segment physical address: {}", segment_physical_address);

        // Calculate the number of pages needed to load the segment
        let num_pages = (segment_size + Size4KiB::SIZE - 1) / Size4KiB::SIZE;

        // Iterate over the pages in the segment
        for page_offset in 0..num_pages {
            let page_virtual_address = segment_virtual_address + page_offset * Size4KiB::SIZE as u64;

            // Get the physical frame for the page
            let frame = PhysFrame::<Size4KiB>::containing_address(PhysAddr::new(segment_physical_address + page_offset * Size4KiB::SIZE as u64));

            // Map the virtual address to the physical frame in the page table
            memory::create_mapping(Page::<Size4KiB>::containing_address(VirtAddr::new(page_virtual_address)), frame, &mut mapper, &mut frame_allocator);
        }
    }

    // TODO: fill in the rest of the memory with unused pages
    let unused_regions = boot_info.memory_map.iter().filter(|r| r.region_type == MemoryRegionType::Usable);

    // TODO: set stack pointer to highest address in page table
    let stack_pointer = VirtAddr::new(0);

    (mapper, frame_allocator, stack_pointer, VirtAddr::new(entry_point))
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