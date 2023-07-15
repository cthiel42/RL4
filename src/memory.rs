use x86_64::{
    structures::paging::{Size4KiB, PhysFrame, Page, PageTable, PageTableFlags, OffsetPageTable, Mapper, FrameAllocator, {mapper::MapToError}},
    PhysAddr,
    VirtAddr,
};
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use bootloader::BootInfo;
use crate::allocator;
use x86_64::instructions::interrupts;

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

struct MemoryInfo {
    boot_info: &'static BootInfo,
    physical_memory_offset: VirtAddr,
    frame_allocator: BootInfoFrameAllocator,
    kernel_l4_table: &'static mut PageTable
}

static mut MEMORY_INFO: Option<MemoryInfo> = None;

pub unsafe fn init(boot_info: &'static BootInfo) {
    interrupts::without_interrupts(|| {
        let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);
        let kernel_l4_table = unsafe {active_level_4_table(physical_memory_offset)};
        let mut mapper = unsafe {OffsetPageTable::new(kernel_l4_table, physical_memory_offset)};
        let mut frame_allocator = unsafe {
            BootInfoFrameAllocator::init(&boot_info.memory_map)
        };

        allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");
        
        unsafe { MEMORY_INFO = Some(MemoryInfo {
            boot_info,
            physical_memory_offset,
            frame_allocator,
            kernel_l4_table
        }) };
    });
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;
    
    let (level_4_table_frame, _) = Cr3::read();
    
    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
    
    unsafe { &mut *page_table_ptr }
}

pub fn create_empty_pagetable() -> (*mut PageTable, u64) {
    let memory_info = unsafe {MEMORY_INFO.as_mut().unwrap()};
    let level_4_table_frame = memory_info.frame_allocator.allocate_frame().unwrap();
    let virtual_address = memory_info.physical_memory_offset + level_4_table_frame.start_address().as_u64();
    let page_table_ptr: *mut PageTable = virtual_address.as_mut_ptr();

    // zero out the page table
    unsafe {(*page_table_ptr).zero();}

    // Return virtual address and physical address to empty page table
    (page_table_ptr, level_4_table_frame.start_address().as_u64())
}

pub fn create_new_user_pagetable() -> (*mut PageTable, u64) {
    // Create a new level 4 pagetable
    let (table_ptr, table_physaddr) = create_empty_pagetable();
    let table = unsafe {&mut *table_ptr};

    fn copy_pages_rec(physical_memory_offset: VirtAddr, from_table: &PageTable, to_table: &mut PageTable, level: u16) {
        for (i, entry) in from_table.iter().enumerate() {
            if !entry.is_unused() {
                if (level == 1) || entry.flags().contains(PageTableFlags::HUGE_PAGE) {
                    // Maps a frame, not a page table
                    to_table[i].set_addr(entry.addr(), entry.flags());
                } else {
                    // Create a new table at level - 1
                    let (new_table_ptr, new_table_physaddr) = create_empty_pagetable();
                    let to_table_m1 = unsafe {&mut *new_table_ptr};

                    // Point the entry to the new table
                    to_table[i].set_addr(PhysAddr::new(new_table_physaddr), entry.flags());

                    // Get reference to the input level-1 table
                    let from_table_m1 = {
                        let virt = physical_memory_offset + entry.addr().as_u64();
                        unsafe {& *virt.as_ptr()}
                    };

                    // Copy level-1 entries
                    copy_pages_rec(physical_memory_offset, from_table_m1, to_table_m1, level - 1);
                }
            }
        }
    }

    // Copy kernel pages
    let memory_info = unsafe {MEMORY_INFO.as_mut().unwrap()};
    copy_pages_rec(memory_info.physical_memory_offset, memory_info.kernel_l4_table, table, 4);

    return (table_ptr, table_physaddr)
}

pub fn allocate_pages_mapper(
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
    mapper: &mut impl Mapper<Size4KiB>,
    start_addr: VirtAddr,
    size: u64,
    flags: PageTableFlags)
    -> Result<(), MapToError<Size4KiB>> {

    let page_range = {
        let end_addr = start_addr + size - 1u64;
        let start_page = Page::containing_address(start_addr);
        let end_page = Page::containing_address(end_addr);
        Page::range_inclusive(start_page, end_page)
    };

    for page in page_range {
        let frame = frame_allocator.allocate_frame().ok_or(MapToError::FrameAllocationFailed)?;
        unsafe {
            mapper.map_to(page, frame, flags, frame_allocator)?.flush()
        };
    }

    Ok(())
}

pub fn allocate_pages(level_4_table: *mut PageTable, start_addr: VirtAddr, size: u64, flags: PageTableFlags) -> Result<(), MapToError<Size4KiB>> {
    let memory_info = unsafe {MEMORY_INFO.as_mut().unwrap()};

    let mut mapper = unsafe {
        OffsetPageTable::new(&mut *level_4_table, memory_info.physical_memory_offset)
    };

    allocate_pages_mapper(
        &mut memory_info.frame_allocator,
        &mut mapper,
        start_addr, size, flags
    )
}

impl BootInfoFrameAllocator {
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        let regions = self.memory_map.iter();
        let usable_regions = regions
            .filter(|r| r.region_type == MemoryRegionType::Usable);
        let addr_ranges = usable_regions
            .map(|r| r.range.start_addr()..r.range.end_addr());
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        frame_addresses.map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}