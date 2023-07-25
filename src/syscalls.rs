use core::arch::asm;
use core::{slice, str, ptr};
use crate::threads;
use crate::cpu;
use crate::arch::arch::RegisterState;

const MSR_STAR: usize = 0xc0000081;
const MSR_LSTAR: usize = 0xc0000082;
const MSR_FMASK: usize = 0xc0000084;
pub const SYSCALL_ERROR_SEND_BLOCKING: u64 = 1;
pub const SYSCALL_ERROR_RECV_BLOCKING: u64 = 2;
pub const SYSCALL_ERROR_INVALID_HANDLE: u64 = 3;

#[naked]
extern "C" fn handle_syscall() {
    unsafe {
        asm!(
            // Switch to kernel stack and backup registers for sysretq
            "push rcx",
            "push r11",
            "push rbp",
            "push rbx", // save callee registers
            "push r12",
            "push r13",
            "push r14",
            "push r15",

            "cmp rax, 0",         // if rax == 0 {
            "jne 1f",
            "call {hello_world}", //   hello_world();
            "jmp 5f",             //   jump to end

            "1: cmp rax, 1",      // } if rax == 1 {
            "jne 3f",
            "call {sys_write}",   //   sys_write();
            "jmp 5f",             //   jump to end

            // "2: cmp rax, 2",      // } if rax == 2 {
            // "jne 3f",
            // "call {ipc_write}",   //   ipc_write();
            // "jmp 5f",             //   jump to end

            "3: cmp rax, 3",      // } if rax == 3 {
            "jne 5f",             //   jump to end since there are no more syscalls
            "call {ipc_read}",    //   ipc_read();
            "jmp 5f",             //   jump to end

            "5: ",                // }

            "pop r15", // restore callee registers
            "pop r14",
            "pop r13",
            "pop r12",
            "pop rbx",
            "pop rbp", // restore stack and registers for sysretq
            "pop r11",
            "pop rcx",
            "sysretq", // back to userspace
            sys_write = sym sys_write,
            hello_world = sym hello_world,
            ipc_read = sym ipc_read,
            options(noreturn)
        );
    }
}

pub fn init() {
    let handler_addr = handle_syscall as *const () as u64;
    unsafe {
        asm!("mov ecx, 0xC0000080",
            "rdmsr",
            "or eax, 1",
            "wrmsr"
        );
        asm!("xor rdx, rdx",
            "mov rax, 0x200",
            "wrmsr",
            in("rcx") MSR_FMASK
        );
        asm!("mov rdx, rax",
            "shr rdx, 32",
            "wrmsr",
            in("rax") handler_addr,
            in("rcx") MSR_LSTAR
        );
        asm!(
            "xor rax, rax",
            "mov rdx, 0x230008",
            "wrmsr",
            in("rcx") MSR_STAR
        );
    }
}

fn ipc_read(context_ptr: *mut RegisterState, handle: u64) {
    // Extract the current thread
    if let Some(mut thread) = threads::take_current_thread() {
        let current_id = thread.id();
        thread.set_context(context_ptr);

        // Get the Rendezvous and call
        if let Some(rdv) = thread.rendezvous(handle) {
            let (thread1, thread2) = rdv.write().receive(thread);
            // thread1 should be started asap
            // thread2 should be scheduled

            let mut returning = false;
            for maybe_thread in [thread1, thread2] {
                if let Some(t) = maybe_thread {
                    if t.id() == current_id {
                        // Same thread -> return
                        threads::set_current_thread(t);
                        returning = true;
                    } else {
                        threads::schedule_thread(t);
                    }
                }
            }

            if !returning {
                // Original thread is waiting. Schedule next thread
                drop(rdv);
                let new_context_addr = threads::schedule_next(context_ptr as usize);
                cpu::launch_thread(new_context_addr);
            }
        } else {
            // Missing handle
            thread.return_error(SYSCALL_ERROR_INVALID_HANDLE);
            threads::set_current_thread(thread);
        }
    }
}

extern "C" fn sys_write(ptr: *mut u8, len:usize) {
    let u8_slice = unsafe {slice::from_raw_parts(ptr, len)};

    if let Ok(s) = str::from_utf8(u8_slice) {
        println!("Write '{}'", s);
        // print current stack address
        let stack_addr: u64;
        unsafe {
            asm!("mov {}, rsp", out(reg) stack_addr);
        }
        println!("Stack address: {:#x}", stack_addr);
    } else {
        println!("Write failed");
    }
}

extern "C" fn hello_world() {
    println!("Hello World");
}

// This is used to deallocate something from memory
// Ownership gets transferred to this function and
// subsequently dropped
pub fn drop<T>(_: T) {}
