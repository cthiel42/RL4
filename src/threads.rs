use crate::arch::arch::RegisterState;
use crate::arch::arch::get_registers;

#[derive(Clone)]
#[allow(dead_code)]
enum ThreadState {
    Running,
    Ready,
    Waiting,
    Done,
}

pub struct Thread {
    id: usize,
    state: ThreadState,
    registers: RegisterState,
    instruction_pointer: usize,
}

impl Thread {

    // Create a new thread
    pub fn new(id: usize, stack: usize, entry_point: extern "C" fn(usize) -> !) -> Thread {
        let mut registers = RegisterState::default();
        registers.rdi = id as u64;
        registers.rsi = stack as u64;
        Thread {
            id: id,
            state: ThreadState::Ready,
            registers: registers,
            instruction_pointer: entry_point as usize,
        }
    }

    // Switch to another thread. 
    pub unsafe fn switch_to(&mut self) {
        self.save_state();
        switch(self.instruction_pointer, self.id, self.registers.clone());
    }

    unsafe fn save_state(&mut self) {
        // use the local arch module to get the current register state
        self.registers = get_registers();

        // save the instruction pointer
        self.instruction_pointer = x86_64::instructions::interrupts::without_interrupts(|| {
            x86_64::registers::control::Cr3::read().0.start_address().as_u64() // ???
        }) as usize;

        // determine and set the thread state
        self.state = match self.state {
            ThreadState::Running => ThreadState::Ready,
            ThreadState::Ready => ThreadState::Running,
            _ => self.state.clone(),
        };
    }

}

unsafe fn switch(pointer: usize, id: usize, registers: RegisterState) {
    // TODO: implement, think this will be a lot of assembly and might be where we implement some sort of scheduler
    // Could make sense to move this to its own module if it gets too big
    // this function shouldnt do anything right now, its just for testing
    println!("Switching to thread {}", id);
}