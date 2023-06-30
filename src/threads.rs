use crate::arch::arch::RegisterState;
use crate::arch::arch::get_registers;
use crate::arch::arch::set_registers;
use crate::memory;
use core::arch::asm;
use x86_64::{structures::paging::{PageTable, PhysFrame, OffsetPageTable}, PhysAddr};
use x86_64::registers::control::{Cr3, Cr3Flags};
use bootloader::BootInfo;

#[derive(Clone)]
#[allow(dead_code)]
enum ThreadState {
    Running,
    Ready,
    Waiting,
    Done,
}

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
        registers.rdi = id;
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
        set_registers(&mut self.registers);

        /*
        // TODO: Implement proper paging and use this
        // Load the page table into the CR3 register
        let mut level_4_table = self.page_table.level_4_table();
        let level_4_table_pointer: u64 = level_4_table as *const _ as u64;
        println!("Level 4 Table Pointer: {:x}", level_4_table_pointer);
        Cr3::write(PhysFrame::containing_address(PhysAddr::new(level_4_table_pointer)), Cr3Flags::empty()); 
        */

        // Jump to the entry point
        println!("Jumping to entry point");
        let entry_point = self.instruction_pointer;
        asm!("jmp {}", in(reg) entry_point);
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
        self.threads[id].instruction_pointer = instruction_pointer;
    }

    pub fn set_stack_pointer(&mut self, id: usize, stack_pointer: u64) {
        self.threads[id].registers.rsi = stack_pointer;
    }

    pub fn get_instruction_pointer(&mut self, id: usize) -> u64 {
        self.threads[id].instruction_pointer
    }

    pub fn get_stack_pointer(&mut self, id: usize) -> u64 {
        self.threads[id].registers.rsi
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