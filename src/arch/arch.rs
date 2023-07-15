use core::arch::asm;

#[derive(Clone, Debug)]
#[repr(C)]
pub struct RegisterState {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rbp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,    
    pub ss: u64, 
}

// Bytes needed to store the RegisterState struct
pub const INTERRUPT_CONTEXT_SIZE: usize = 20 * 8;

impl Default for RegisterState {
    fn default() -> RegisterState {
        RegisterState {
            rax: 0, rbx: 0, rcx: 0, rdx: 0, rsi: 0, rsp: 0, rdi: 0, rbp: 0, r8: 0, r9: 0, r10: 0, r11: 0, r12: 0, r13: 0, r14: 0, r15: 0, rip: 0, cs: 0, rflags: 0, ss: 0,
        }
    }
}

pub fn get_cr3() -> u64 {
    let cr3: u64;
    unsafe {
        asm!("mov {}, cr3", out(reg) cr3);
    }
    cr3
}

pub fn set_cr3(physaddr: u64) {
    unsafe {
        asm!("mov cr3, {addr}", addr = in(reg) physaddr);
    }
}

pub unsafe fn get_registers() -> RegisterState {
    let mut register = RegisterState::default();
    asm!("mov {}, rax", out(reg) register.rax);
    asm!("mov {}, rbx", out(reg) register.rbx);
    asm!("mov {}, rcx", out(reg) register.rcx);
    asm!("mov {}, rdx", out(reg) register.rdx);
    asm!("mov {}, rsi", out(reg) register.rsi);
    asm!("mov {}, rsp", out(reg) register.rsp);
    asm!("mov {}, rdi", out(reg) register.rdi);
    asm!("mov {}, rbp", out(reg) register.rbp);
    asm!("mov {}, r8", out(reg) register.r8);
    asm!("mov {}, r9", out(reg) register.r9);
    asm!("mov {}, r10", out(reg) register.r10);
    asm!("mov {}, r11", out(reg) register.r11);
    asm!("mov {}, r12", out(reg) register.r12);
    asm!("mov {}, r13", out(reg) register.r13);
    asm!("mov {}, r14", out(reg) register.r14);
    asm!("mov {}, r15", out(reg) register.r15);
    register
}

pub unsafe fn set_registers(register: &mut RegisterState, instruction_pointer: u64) {
    // asm!("mov rax, {}", in(reg) register.rax);
    // asm!("mov rbx, {}", in(reg) register.rbx);
    // asm!("mov rcx, {}", in(reg) register.rcx);
    // asm!("mov rdx, {}", in(reg) register.rdx);
    // asm!("mov rsi, {}", in(reg) register.rsi);
    // asm!("mov rdi, {}", in(reg) register.rdi);
    // asm!("mov rbp, {}", in(reg) register.rbp);
    // asm!("mov r8, {}", in(reg) register.r8);
    // asm!("mov r9, {}", in(reg) register.r9);
    // asm!("mov r10, {}", in(reg) register.r10);
    // asm!("mov r11, {}", in(reg) register.r11);
    // asm!("mov r12, {}", in(reg) register.r12);
    // asm!("mov r13, {}", in(reg) register.r13);
    // asm!("mov r14, {}", in(reg) register.r14);
    // asm!("mov r15, {}", in(reg) register.r15);

    asm!(
        "mov rsp, {0}",
        "jmp {1}",
        in(reg) register.rsp,
        in(reg) instruction_pointer,
    );
}