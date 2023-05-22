# Memory Management

## Paging

Our kernel needs to be able to isolate processes from one another. Part of that is ensuring each process has it's own memory to use. Memory Paging is a method where we can break apart our physical memory into smaller chunks, called `pages`, that is then used by a process. A big piece of making this work effectively is using virtual memory addresses to abstract away the fact that the memory is not actually a sequential chunk of physical memory. A translation is done by the kernel to get from the virtual memory to the physical memory. Each page of virtual memory is mapped to the physical memory, which is called a `frame`. This mapping between pages and frames needs to be stored somewhere, which is what a `page table` is for. Each process running on the kernel will have it's own page table. A pointer to the currently active page table is stored in it's own CPU register called `CR3`. The kernel needs to load this register with the correct page table pointer for a process before that process is ran, since the CPU will reference this table when accessing the memory. 

Multilevel or hierarchical page tables can also be used to cut down on the size of tables that may have large blocks of unallocated memory. The first page table interacted with in this case would have much larger page sizes, and instead of returning frames, it would point to another more fine grained page table that would then return the frame. The x86_64 system uses a 4 level page table and a page size of 4 KiB. Each page table has a fixed size of 512 mappings. These sizing numbers were strategic, as this makes each table fit exactly into one page. Each page table is essentially just an array with 512 entries. A Rust representation would look like this:

    #[repr(align(4096))]
    pub struct PageTable {
        entries: [PageTableEntry; 512],
    }

The bootloader crate currently in use actually initializes the page tables. This is out of necessity as a part of the boot process. If I have time down the road, I still think building the relevant bits from the bootloader directly into the kernel and removing the dependency on the bootloader crate would be a worthwhile task. We can enable a feature in the bootloader that maps all of the physical memory to a virtual address range that the kernel can then use. The BootInfo struct that gets passed into our `_start` function will then have information on the available physical memory and the offset at which the physical memory mappings start in the virtual memory provided to us.

The memory management code lives in `memory.rs`. To avoid too much code cluttering this document, I'll describe some of the functions used without putting all of the code here.

We'll start by creating an `init` function to create the page table that we'll use. It'll call a helper function, `active_level_4_table`, to read from the `CR3` register to get the physical starting address, then do the offset using information from the boot info that gets passed through the init function. The `active_level_4_table` ends up being an unsafe function due to the memory pointers.

From there we need to be able to do translations, which can be abstracted to the `x86_64` crate using the `translate_addr` function. That should be all that's needed to read from the page tables. Adding an entry to the page table is another story. We create a function called `create_mapping` which takes the page and frame to map, the offset page table we're using, and a frame allocator. The frame allocator is used in the case that a new page table needs to be created. Since page tables are initialized during boot, we end up having to create our own frame allocator from the boot info. We call it the `BootInfoFrameAllocator`. The struct for it looks like this:

    pub struct BootInfoFrameAllocator {
        memory_map: &'static MemoryMap,
        next: usize,
    }

We also create an implementation for it that has an `init` function to set the memory map using the map from our boot info. The memory map is a list of `MemoryRegion` structs, which contain information on the start address, size, and current use of the memory. The `next` field keeps track of the number of the next frame that should be returned, so the init function starts it at 0. The init function ends up being unsafe since it requires the caller to guarantee that the usable frames provided in the memory map weren't already used somewhere else.

In order to implement our `BootInfoFrameAllocator` into a `FrameAllocator`, we create a function called `usable_frames` that returns an iterator over the usable frames in the memory map provided to the boot info frame allocator. We can then implement the FrameAllocator as such.

    unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
        fn allocate_frame(&mut self) -> Option<PhysFrame> {
            let frame = self.usable_frames().nth(self.next);
            self.next += 1;
            frame
        }
    }

Some code was also added to the `_start` function in order to test the functionality of all of this. This will likely get taken out in short order, so I'll provide all of it here. First we can verify that we can read the page table by looking for values we expect.

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

The main takeaways here are that the VGA buffer page is always the same address, so the virtual and physical addresses should be the same. The physical memory offset is also a good way of checking that our offset is indeed correct. From there we can test creating a new mapping and page table.

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    // map an unused but existing page to the vga buffer
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

Our first mapping shows that our mappings are indeed working, since we can write to the VGA buffer and see it when we run the kernel. Our second mapping is one that requires a new page table to be created, so as long as we don't see an error we're good.

More information on memory paging can be found [here](https://os.phil-opp.com/paging-introduction/) and [here](https://os.phil-opp.com/paging-implementation/)



