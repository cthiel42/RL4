extern crate alloc;
use alloc::vec::Vec;
use spin::RwLock;
use lazy_static::lazy_static;
use alloc::{boxed::Box, collections::vec_deque::VecDeque};
use x86_64::instructions::interrupts;
use x86_64::VirtAddr;
use x86_64::structures::paging::PageTableFlags;
use crate::gdt;
use crate::memory;
use crate::arch::arch::{RegisterState,INTERRUPT_CONTEXT_SIZE};

const KERNEL_STACK_SIZE: usize = 4096 * 2;
const USER_STACK_SIZE: usize = 4096 * 5;
pub const USER_CODE_START: u64 = 0x2000000;
pub const USER_CODE_END: u64 = 0x5000000;
const USER_STACK_START: u64 = 0x3000000;
const USER_HEAP_START: u64 = 0x280_0060_0000;
const USER_HEAP_SIZE: u64 = 4 * 1024 * 1024; 

struct Thread {
    kernel_stack: Vec<u8>,
    user_stack: Vec<u8>,
    kernel_stack_end: u64, // goes in TSS
    user_stack_end: u64,
    context: u64, // Address of register state on kernel stack
}

lazy_static! {
    static ref RUNNING_QUEUE: RwLock<VecDeque<Box<Thread>>> =
        RwLock::new(VecDeque::new());

    static ref CURRENT_THREAD: RwLock<Option<Box<Thread>>> =
        RwLock::new(None);
}

pub fn new_kernel_thread(function: fn()->()) {
    let new_thread = {
        let kernel_stack = Vec::with_capacity(KERNEL_STACK_SIZE);
        let kernel_stack_end = (VirtAddr::from_ptr(kernel_stack.as_ptr()) + KERNEL_STACK_SIZE).as_u64();
        let user_stack = Vec::with_capacity(USER_STACK_SIZE);
        let user_stack_end = (VirtAddr::from_ptr(user_stack.as_ptr()) + USER_STACK_SIZE).as_u64();
        let context = kernel_stack_end - INTERRUPT_CONTEXT_SIZE as u64;

        Box::new(Thread {
            kernel_stack,
            user_stack,
            kernel_stack_end,
            user_stack_end,
            context})
    };

    // Set context registers
    let context = unsafe {&mut *(new_thread.context as *mut RegisterState)};
    context.rip = function as u64;              // Instruction pointer
    context.rsp = new_thread.user_stack_end;    // Stack pointer
    context.rflags = 0x200;                     // Interrupts enabled

    let (code_selector, data_selector) = gdt::get_kernel_segments();
    context.cs = code_selector.0 as u64;
    context.ss = data_selector.0 as u64;

    // Add Thread to RUNNING_QUEUE
    interrupts::without_interrupts(|| {
        RUNNING_QUEUE.write().push_back(new_thread);
    });
}

pub fn schedule_next(context: &RegisterState) -> usize {
    let mut running_queue = RUNNING_QUEUE.write();
    let mut current_thread = CURRENT_THREAD.write();

    if let Some(mut thread) = current_thread.take() {
        let mut proc_mut = thread;
        proc_mut.context = (context as *const RegisterState) as u64;
        running_queue.push_back(proc_mut);
    }
    // Get the next thread in the queue
    *current_thread = running_queue.pop_front();
    match current_thread.as_ref() {
        Some(thread) => {
            // Set the kernel stack for the next interrupt
            gdt::set_interrupt_stack_table(
              gdt::TIMER_INTERRUPT_INDEX as usize,
              VirtAddr::new(thread.kernel_stack_end));
            // Point the stack to the new context
            thread.context as usize
          },
        None => 0  // Timer handler won't modify stack
    }
}

pub fn new_user_thread(bin: &[u8]) -> Result<usize, &'static str> {
    use elf::endian::AnyEndian;
    use elf::ElfBytes;
    use elf::abi::PT_LOAD;

    // Verify headers are for an ELF file
    const ELF_HEADERS: [u8; 4] = [0x7f, b'E', b'L', b'F'];
    if bin[0..4] != ELF_HEADERS {
        return Err("Invalid ELF file");
    }

    let file = ElfBytes::<AnyEndian>::minimal_parse(bin).unwrap();
    let entry_point: u64 = file.ehdr.e_entry;
    let user_page_table_ptr = memory::create_user_pagetable();

    for segment in file.segments().unwrap().iter() {
        // println!("Segment: {:?}", segment);
        if segment.p_type != PT_LOAD {
            continue;
        }
        let segment_address = segment.p_vaddr as u64;
        let segment_size = segment.p_memsz as u64;
        let start_address = VirtAddr::new(segment_address);
        let end_address = start_address + segment_size;
        if (start_address < VirtAddr::new(USER_CODE_START))
            || (end_address >= VirtAddr::new(USER_CODE_END)) {
                return Err("ELF segment outside allowed range");
            }

        // Allocate memory in the pagetable
        if memory::allocate_pages(user_page_table_ptr,
                            VirtAddr::new(segment_address),
                            segment_size,
                            PageTableFlags::PRESENT |
                            PageTableFlags::WRITABLE |
                            PageTableFlags::USER_ACCESSIBLE).is_err() {
            return Err("Could not allocate memory");
        }

        let source = &bin[segment.p_offset as usize..][..segment.p_filesz as usize];
        crate::arch::elf::copy_memory(segment_address as usize, source);
    }

    let mut new_thread = {
        let kernel_stack = Vec::with_capacity(KERNEL_STACK_SIZE);
        let kernel_stack_end = (VirtAddr::from_ptr(kernel_stack.as_ptr()) + KERNEL_STACK_SIZE).as_u64();
        let user_stack = Vec::with_capacity(USER_STACK_SIZE);
        let user_stack_end = (VirtAddr::from_ptr(user_stack.as_ptr()) + USER_STACK_SIZE).as_u64();
        let context = kernel_stack_end - INTERRUPT_CONTEXT_SIZE as u64;

        Box::new(Thread {
            kernel_stack,
            user_stack,
            kernel_stack_end,
            user_stack_end,
            context
        })
    };

    let context = unsafe {&mut *(new_thread.context as *mut RegisterState)};
    context.rip = entry_point;

    let (code_selector, data_selector) = gdt::get_user_segments();
    context.cs = code_selector.0 as u64;
    context.ss = data_selector.0 as u64;

    if memory::allocate_pages(user_page_table_ptr,
                           VirtAddr::new(USER_STACK_START),
                           USER_STACK_SIZE as u64,
                           PageTableFlags::PRESENT |
                           PageTableFlags::WRITABLE |
                           PageTableFlags::USER_ACCESSIBLE).is_err()  {
        return Err("Could not allocate memory");
    }
    context.rsp = (USER_STACK_START as u64) + USER_STACK_SIZE as u64;
    
    println!("Adding user thread to queue");
    interrupts::without_interrupts(|| {
        RUNNING_QUEUE.write().push_back(new_thread);
    });

    return Ok(0);
}