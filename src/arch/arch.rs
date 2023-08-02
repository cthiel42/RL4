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