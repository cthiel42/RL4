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
    frame_allocator: BootInfoFrameAllocator
}

static mut MEMORY_INFO: Option<MemoryInfo> = None;

pub unsafe fn init(boot_info: &'static BootInfo) {
    interrupts::without_interrupts(|| {
        let physical_memory_offset = VirtAddr::new(boot_info.physical_memory_offset);
        let level_4_table = unsafe {active_level_4_table(physical_memory_offset)};
        let mut mapper = unsafe {OffsetPageTable::new(level_4_table, physical_memory_offset)};
        let mut frame_allocator = unsafe {
            BootInfoFrameAllocator::init(&boot_info.memory_map)
        };

        allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");
        
        unsafe { MEMORY_INFO = Some(MemoryInfo {
            boot_info,
            physical_memory_offset,
            frame_allocator
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

pub fn create_user_pagetable() -> *mut PageTable {
    let memory_info = unsafe {MEMORY_INFO.as_mut().unwrap()};
    let table = unsafe {active_level_4_table(memory_info.physical_memory_offset)};
    table as *mut PageTable
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