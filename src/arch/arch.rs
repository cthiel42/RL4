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
    let mut register = RegisterState::default();
    asm!("mov rax, {}", out(reg) register.rax);
    asm!("mov rbx, {}", out(reg) register.rbx);
    asm!("mov rcx, {}", out(reg) register.rcx);
    asm!("mov rdx, {}", out(reg) register.rdx);
    asm!("mov rsi, {}", out(reg) register.rsi);
    asm!("mov rdi, {}", out(reg) register.rdi);
    asm!("mov rbp, {}", out(reg) register.rbp);
    asm!("mov r8, {}", out(reg) register.r8);
    asm!("mov r9, {}", out(reg) register.r9);
    asm!("mov r10, {}", out(reg) register.r10);
    asm!("mov r11, {}", out(reg) register.r11);
    asm!("mov r12, {}", out(reg) register.r12);
    asm!("mov r13, {}", out(reg) register.r13);
    asm!("mov r14, {}", out(reg) register.r14);
    asm!("mov r15, {}", out(reg) register.r15);
    register
}