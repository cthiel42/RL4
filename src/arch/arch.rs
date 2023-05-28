use core::arch::asm;

#[derive(Clone)]
pub struct RegisterState {
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub rbp: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
}

impl Default for RegisterState {
    fn default() -> RegisterState {
        RegisterState {
            rax: 0, rbx: 0, rcx: 0, rdx: 0, rsi: 0, rdi: 0, r8: 0, r9: 0, r10: 0, r11: 0, r12: 0, r13: 0, r14: 0, r15: 0, rbp: 0,
        }
    }
}

pub unsafe fn get_registers() -> RegisterState {
    // TODO: Figure out how to get the registers and return them in a RegisterState struct
    asm!("mov rax, rax");

    RegisterState::default()
}