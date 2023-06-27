use x86_64::{
    structures::paging::{Size4KiB, PhysFrame, Page, PageTable, OffsetPageTable, Mapper, FrameAllocator, RecursivePageTable},
    PhysAddr,
    VirtAddr,
};
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level_4_table = active_level_4_table(physical_memory_offset);
    OffsetPageTable::new(level_4_table, physical_memory_offset)
}

pub fn new_page_table() -> PageTable {
    PageTable::new()
}

pub fn new_mapper<'a>(page_table: &'a mut PageTable) -> OffsetPageTable<'a> {
    let mapper = unsafe { OffsetPageTable::new(page_table, VirtAddr::new(0)) };
    mapper
}

unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;
    
    let (level_4_table_frame, _) = Cr3::read();
    
    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
    
    unsafe { &mut *page_table_ptr }
}

pub fn remove_mapping(page: Page, mapper: &mut OffsetPageTable) {
    let unmap_result = unsafe {
        mapper.unmap(page)
    };
    unmap_result.expect("unmap failed").1.flush();
}

pub fn create_mapping(
    page: Page,
    frame: PhysFrame,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;
    let flags = Flags::PRESENT | Flags::WRITABLE | Flags::USER_ACCESSIBLE;
    let map_to_result = unsafe {
        // Possibly lets a frame be mapped to multiple pages. TODO: Test this theory
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("map_to failed").flush();
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