extern crate alloc;
use alloc::vec::Vec;
use spin::RwLock;
use lazy_static::lazy_static;
use alloc::{boxed::Box, collections::vec_deque::VecDeque, sync::Arc};
use core::arch::asm;
use x86_64::instructions::interrupts;
use x86_64::VirtAddr;
use x86_64::structures::paging::PageTableFlags;
use crate::gdt;
use crate::memory;
use crate::arch::arch::{RegisterState, INTERRUPT_CONTEXT_SIZE, set_cr3, get_cr3};
use crate::ipc::{Message,Rendezvous};

const KERNEL_STACK_SIZE: usize = 4096 * 2;
const USER_STACK_SIZE: usize = 4096 * 5;
pub const USER_CODE_START: u64 = 0x2000000;
pub const USER_CODE_END: u64 = 0x5000000;
const USER_STACK_START: u64 = 0x3000000;
const USER_HEAP_START: u64 = 0x280_0060_0000;
const USER_HEAP_SIZE: u64 = 4 * 1024 * 1024; 

pub struct Thread {
    id: u64,
    handles: Vec<Arc<RwLock<Rendezvous>>>,
    kernel_stack: Vec<u8>,
    user_stack: Vec<u8>,
    kernel_stack_end: u64,
    user_stack_end: u64,
    context: u64, // Address of register state on kernel stack
    page_table_physaddr: u64
}

lazy_static! {
    static ref RUNNING_QUEUE: RwLock<VecDeque<Box<Thread>>> = RwLock::new(VecDeque::new());
    static ref CURRENT_THREAD: RwLock<Option<Box<Thread>>> = RwLock::new(None);
    static ref THREAD_COUNTER: RwLock<u64> = RwLock::new(0);
}

pub fn new_kernel_thread(function: fn()->()) {
    let new_thread = {
        let kernel_stack = Vec::with_capacity(KERNEL_STACK_SIZE);
        let kernel_stack_end = (VirtAddr::from_ptr(kernel_stack.as_ptr()) + KERNEL_STACK_SIZE).as_u64();
        let user_stack = Vec::with_capacity(USER_STACK_SIZE);
        let user_stack_end = (VirtAddr::from_ptr(user_stack.as_ptr()) + USER_STACK_SIZE).as_u64();
        let context = kernel_stack_end - INTERRUPT_CONTEXT_SIZE as u64;

        Box::new(Thread {
            id: next_id(),
            handles: Vec::new(),
            kernel_stack,
            user_stack,
            kernel_stack_end,
            user_stack_end,
            context,
            page_table_physaddr: 0})
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

pub fn schedule_next(context_addr: usize) -> usize {
    let mut running_queue = RUNNING_QUEUE.write();
    let mut current_thread = CURRENT_THREAD.write();

    if let Some(mut thread) = current_thread.take() {
        thread.context = context_addr as u64;
        thread.page_table_physaddr = get_cr3();
        running_queue.push_back(thread);
    }
    
    // Get the next thread in the queue
    *current_thread = running_queue.pop_front();
    match current_thread.as_ref() {
        Some(thread) => {
            // Set the kernel stack for the next interrupt
            gdt::set_interrupt_stack_table(
              gdt::TIMER_INTERRUPT_INDEX as usize,
              VirtAddr::new(thread.kernel_stack_end));
            if thread.page_table_physaddr != 0 {
                set_cr3(thread.page_table_physaddr);
            }
            // println!("Switching to thread {}", thread.id());
            // Point the stack to the new context
            thread.context as usize
          },
        None => 0  // Timer handler won't modify stack
    }
}

pub fn new_user_thread(bin: &[u8], handles: Vec<Arc<RwLock<Rendezvous>>>) -> Result<usize, &'static str> {
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
    let (user_page_table_ptr, user_page_table_physaddr) = memory::create_new_user_pagetable();
    set_cr3(user_page_table_physaddr);

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
            id: next_id(),
            handles,
            kernel_stack,
            user_stack,
            kernel_stack_end,
            user_stack_end,
            context,
            page_table_physaddr: user_page_table_physaddr
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
    context.rax = USER_HEAP_START as u64;
    context.rcx = USER_HEAP_SIZE as u64;
    
    println!("Adding user thread to queue");
    interrupts::without_interrupts(|| {
        RUNNING_QUEUE.write().push_front(new_thread);
    });

    return Ok(0);
}

pub fn next_id() -> u64 {
    interrupts::without_interrupts(|| {
        let mut counter = THREAD_COUNTER.write();
        *counter += 1;
        *counter
    })
}

pub fn take_current_thread() -> Option<Box<Thread>> {
    CURRENT_THREAD.write().take()
}

// Add thread to beginning of queue
pub fn schedule_thread(thread: Box<Thread>) {
    // Turn off interrupts while modifying process table
    interrupts::without_interrupts(|| {
        RUNNING_QUEUE.write().push_front(thread);
    });
}

// Makes the given thread the current thread
pub fn set_current_thread(thread: Box<Thread>) {
    // Replace the current thread
    let old_current = CURRENT_THREAD.write().replace(thread);
    if let Some(t) = old_current {
        schedule_thread(t);
    }
}

impl Thread {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn rendezvous(&self, id: u64) -> Option<Arc<RwLock<Rendezvous>>> {
        self.handles.get(id as usize).map(|rv| rv.clone())
    }

    fn context_mut(&self) -> &mut RegisterState {
        unsafe {&mut *(self.context as *mut RegisterState)}
    }

    fn context(&self) -> &RegisterState {
        unsafe {& *(self.context as *const RegisterState)}
    }

    pub fn set_context(&mut self, context_ptr: *mut RegisterState) {
        self.context = context_ptr as u64;
    }

    pub fn return_error(&self, error_code: u64) {
        self.context_mut().rax = error_code;
    }

    pub fn return_message(&self, message: Message) {
        let context = self.context_mut();
        context.rax = 0;
        match message {
            Message::Short(value) => {
                context.rdi = value;
            },
            Message::Long => {
                context.rdi = 42;
            }
        }
    }
}