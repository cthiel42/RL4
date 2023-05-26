

enum ThreadState {
    Running,
    Ready,
    Waiting,
    Done,
}

#[derive(Clone)]
struct RegisterState {
    rax: u64,
    rbx: u64,
    rcx: u64,
    rdx: u64,
    rsi: u64,
    rdi: u64,
    rbp: u64,
    r8: u64,
    r9: u64,
    r10: u64,
    r11: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,
}

impl Default for RegisterState {
    fn default() -> RegisterState {
        RegisterState {
            rax: 0, rbx: 0, rcx: 0, rdx: 0, rsi: 0, rdi: 0, r8: 0, r9: 0, r10: 0, r11: 0, r12: 0, r13: 0, r14: 0, r15: 0, rbp: 0,
        }
    }
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
        switch(self.instruction_pointer, self.id, self.registers.clone()); // TODO: implement, think this will be a lot of assembly

    }
}

unsafe fn switch(pointer: usize, id: usize, registers: RegisterState) {
    // this function shouldnt do anything right now, its just for testing
    println!("Switching to thread {}", id);
}