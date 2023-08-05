# Heap Allocation

## Variables

As of now the kernel currently supports local variables and static variables. Local variables are scoped to a function, live on the call stack, and are deallocated on function return. Static variables live for the life of the program. Things get slightly more complicated if we want to dynamically create variables and pass them between functions. We can really only do this right now using static variables, which defeats the purpose of dynamic allocation. So the solution is to set up a heap, which will allow us to allocate variables into a memory range. Then our variables can act as pointers to the address it exists in memory.

## Heap

For the most part we really need to set up the memory range the heap will use. This is all defined in `allocator.rs`. We'll set two static variables that are used to define the heap location and the heap size. This are relatively arbitrary values. I used this location (other than it's unused memory) since it lets me easily identify what memory is being used for what when I'm troubleshooting. The heap size lets us use 1000 pages worth of memory as well.

    pub const HEAP_START: usize = 0x8000000;
    pub const HEAP_SIZE: usize = 1000 * 4096; // 4 MiB

Then we can start setting up the heap. First we can import everything we need.

    use x86_64::{structures::paging::{mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB},VirtAddr};
    use linked_list_allocator::LockedHeap;

The `linked_list_allocator` is an external dependency that can be added in the `Cargo.toml` file with the line `linked_list_allocator = "0.9.0"`. Then we can create the object that will actually reference and manage the heap for us.

    #[global_allocator]
    static ALLOCATOR: LockedHeap = LockedHeap::empty();

The `#[global_allocator]` flag used here is part of a nightly feature in Rust, so we need to enable it. In `.cargo/config.toml` we have our `[unstable]` block with a `build-std`. We need to specify to recompile `alloc` for our use like so: `build-std = ["core", "compiler_builtins", "alloc"]`.

Then we can just make an init function that allocates our heap in memory.

    pub fn init_heap(mapper: &mut impl Mapper<Size4KiB>, frame_allocator: &mut impl FrameAllocator<Size4KiB>,) -> Result<(), MapToError<Size4KiB>> {
        let page_range = {
            let heap_start = VirtAddr::new(HEAP_START as u64);
            let heap_end = heap_start + HEAP_SIZE - 1u64;
            let heap_start_page = Page::containing_address(heap_start);
            let heap_end_page = Page::containing_address(heap_end);
            Page::range_inclusive(heap_start_page, heap_end_page)
        };

        for page in page_range {
            let frame = frame_allocator
                .allocate_frame()
                .ok_or(MapToError::FrameAllocationFailed)?;
            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
            unsafe {
                mapper.map_to(page, frame, flags, frame_allocator)?.flush()
            };
        }

        unsafe {
            ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
        }

        Ok(())
    }

This function is fairly straightforward. It allocates the required pages in memory and initializes the allocator object. It makes the most logical sense to allocate the heap when we allocate the memory, so this init function is called from within the memory init function using the already available mapper and frame allocator.

    allocator::init_heap(&mut mapper, &mut frame_allocator)

From there if we wanted to allocate something using the heap, we can wrap it with `Box` or use a predefined object from the `alloc` crate. We'll see some of this later on when we start using threads, but a basic example would look like the following:

    extern crate alloc;
    use alloc::vec::Vec;

    let vector = Vec::from(["a", "b", "c"]);

Or using `Box`:

    extern crate alloc;
    use alloc::boxed::Box;

    let allocated_struct = Box::new(Struct {
                    attribute_1,
                    attribute_2,
                    attribute_3}
                );


