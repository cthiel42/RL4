// use crate::arch::arch::RegisterState;
// use crate::arch::arch::get_registers;
// use crate::arch::arch::set_registers;
// use crate::memory;
// use core::arch::asm;
// use x86_64::{structures::paging::{PageTable, PhysFrame, OffsetPageTable}, PhysAddr};
// use x86_64::registers::control::{Cr3, Cr3Flags};
// use bootloader::BootInfo;

extern crate alloc;
use alloc::vec::Vec;
use spin::RwLock;
use lazy_static::lazy_static;
use alloc::{boxed::Box, collections::vec_deque::VecDeque};
use x86_64::instructions::interrupts;
use x86_64::VirtAddr;
use crate::gdt;
use crate::arch::arch::{RegisterState,INTERRUPT_CONTEXT_SIZE};

// #[derive(Clone)]
// #[allow(dead_code)]
// enum ThreadState {
//     Running,
//     Ready,
//     Waiting,
//     Done,
// }

const KERNEL_STACK_SIZE: usize = 4096 * 2;
const USER_STACK_SIZE: usize = 4096 * 5;

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

/*
pub struct Thread {
    id: u64,
    state: ThreadState,
    registers: RegisterState,
    instruction_pointer: u64,
    page_table: OffsetPageTable<'static>,
    frame_allocator: memory::BootInfoFrameAllocator,
}

impl Thread {

    // Create a new thread
    pub fn new(id: u64, stack: u64, entry_point: u64, boot_info: &'static BootInfo, page_table: &'static mut PageTable) -> Thread {
        let mut registers = RegisterState::default();
        registers.rsp = stack;
        let mut mapper = unsafe { memory::new_mapper(page_table) };
        let mut frame_allocator = unsafe {
            memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
        };
        Thread {
            id: id,
            state: ThreadState::Ready,
            registers: registers,
            instruction_pointer: entry_point,
            page_table: mapper,
            frame_allocator: frame_allocator,
        }
    }

    // Switch to this thread
    pub unsafe fn switch_to(&mut self) {
        // Set registers
        println!("Setting registers");
        // println!("Stack pointer: {:x}", self.registers.rsp);
        // println!("Instruction pointer: {:x}", self.instruction_pointer);
        set_registers(&mut self.registers, self.instruction_pointer);

        /*
        // TODO: Implement proper paging and use this
        // Load the page table into the CR3 register
        let mut level_4_table = self.page_table.level_4_table();
        let level_4_table_pointer: u64 = level_4_table as *const _ as u64;
        println!("Level 4 Table Pointer: {:x}", level_4_table_pointer);
        Cr3::write(PhysFrame::containing_address(PhysAddr::new(level_4_table_pointer)), Cr3Flags::empty()); 
        */

        // println!("Jumping to entry point");
        // asm!("jmp {}", in(reg) self.instruction_pointer);
    }


    unsafe fn save_state(&mut self) {
        // use the local arch module to get the current register state
        self.registers = get_registers();

        // save the instruction pointer
        self.instruction_pointer = x86_64::instructions::interrupts::without_interrupts(|| {
            x86_64::registers::control::Cr3::read().0.start_address().as_u64() // ???
        });

        // determine and set the thread state
        self.state = match self.state {
            ThreadState::Running => ThreadState::Ready,
            ThreadState::Ready => ThreadState::Running,
            _ => self.state.clone(),
        };
    }

    pub fn set_stack_pointer(&mut self, stack_pointer: u64) {
        self.registers.rsp = stack_pointer;
    }

    pub fn get_stack_pointer(&mut self) -> u64 {
        self.registers.rsp
    }

    pub fn set_instruction_pointer(&mut self, instruction_pointer: u64) {
        self.instruction_pointer = instruction_pointer;
    }

    pub fn get_instruction_pointer(&mut self) -> u64 {
        self.instruction_pointer
    }
}

pub struct ThreadManager {
    threads: [Thread; 3]
}

impl ThreadManager {

    // Create a new thread manager
    pub fn new(boot_info: &'static BootInfo) -> ThreadManager {
        // TODO: Can make this more safe by getting rid of static mut and maybe using something like lazy_static
        static mut KERNEL_TABLE: PageTable = PageTable::new(); // TODO: Read and load in the actual kernel table
        static mut PAGE_TABLE_1: PageTable = PageTable::new();
        static mut PAGE_TABLE_2: PageTable = PageTable::new();
        ThreadManager {
            threads: unsafe {[
                Thread::new(0, 0, 0, boot_info, &mut KERNEL_TABLE),
                Thread::new(1, 0, 0, boot_info, &mut PAGE_TABLE_1),
                Thread::new(2, 0, 0, boot_info, &mut PAGE_TABLE_2),
            ]}
        }
    }

    pub fn get_thread(&mut self, id: usize) -> &mut Thread {
        &mut self.threads[id]
    }

    pub fn set_thread(&mut self, id: usize, thread: Thread) {
        self.threads[id] = thread;
    }

    pub fn set_instruction_pointer(&mut self, id: usize, instruction_pointer: u64) {
        self.threads[id].set_instruction_pointer(instruction_pointer);
    }

    pub fn set_stack_pointer(&mut self, id: usize, stack_pointer: u64) {
        self.threads[id].set_stack_pointer(stack_pointer);
    }

    pub fn get_instruction_pointer(&mut self, id: usize) -> u64 {
        self.threads[id].get_instruction_pointer()
    }

    pub fn get_stack_pointer(&mut self, id: usize) -> u64 {
        self.threads[id].get_stack_pointer()
    }

    pub fn get_page_table(&mut self, id: usize) -> (&mut OffsetPageTable<'static>, &mut memory::BootInfoFrameAllocator) {
        (&mut self.threads[id].page_table, &mut self.threads[id].frame_allocator)
    }

    pub fn switch_to(&mut self, id: usize) {
        unsafe {
            self.threads[id].switch_to();
        }
    }
}
*/