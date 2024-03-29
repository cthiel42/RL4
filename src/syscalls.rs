use core::arch::asm;
use core::{slice, str};
use crate::threads;
use crate::cpu;
use crate::gdt;
use crate::arch::arch::RegisterState;
use crate::ipc::Message;

const MSR_STAR: usize = 0xc0000081;
const MSR_LSTAR: usize = 0xc0000082;
const MSR_FMASK: usize = 0xc0000084;
const MSR_KERNEL_GS_BASE: usize = 0xC0000102;
pub const SYSCALL_ERROR_INVALID_HANDLE: u64 = 3;
const SYSCALL_KERNEL_STACK_OFFSET: u64 = 1024;

#[naked]
extern "C" fn handle_syscall() {
    unsafe {
        asm!(
            "swapgs",
            "mov gs:{tss_temp}, rsp",
            "mov rsp, gs:{tss_timer}",
            "sub rsp, {ks_offset}",
            "push gs:{tss_temp}",
            "swapgs",

            "push r11",
            "sub rsp, 8",
            "push rcx",
            
            "push rax",
            "push rbx",
            "push rcx",
            "push rdx",
            "push rdi",
            "push rsi",
            "push rbp",
            "push r8",
            "push r9",
            "push r10",
            "push r11",
            "push r12",
            "push r13",
            "push r14",
            "push r15",

            "mov r8, rdx", // Fifth argument <- Syscall third argument
            "mov rcx, rsi", // Fourth argument <- Syscall second argument
            "mov rdx, rdi", // Third argument <- Syscall first argument
            "mov rsi, rax", // Second argument is the syscall number
            "mov rdi, rsp", // First argument is the Context address
            "call {syscall_router}",

            "pop r15", // restore callee-saved registers
            "pop r14",
            "pop r13",
            "pop r12",
            "pop r11",
            "pop r10",
            "pop r9",
            "pop r8",
            "pop rbp",
            "pop rsi",
            "pop rdi",
            "pop rdx",
            "pop rcx",
            "pop rbx",
            "pop rax",

            "add rsp, 24",
            "pop rsp",

            "cmp rcx, {user_code_start}",
            "jl 9f",
            "cmp rcx, {user_code_end}",
            "jge 9f",
            "sysretq", // back to userspace
            
            "9:",
            "push r11",
            "popf",
            "jmp rcx",

            syscall_router = sym syscall_router,
            tss_timer = const(0x24 + gdt::TIMER_INTERRUPT_INDEX * 8),
            tss_temp = const(0x24 + gdt::SYSCALL_TEMP_INDEX * 8),
            ks_offset = const(SYSCALL_KERNEL_STACK_OFFSET),
            user_code_start = const(threads::USER_CODE_START),
            user_code_end = const(threads::USER_CODE_END),
            options(noreturn),
        );
    }
}

extern "C" fn syscall_router(context_ptr: *mut RegisterState, syscall_id: u64, arg1: u64, arg2: u64, arg3: u64) {

    let context = unsafe{&mut *context_ptr};

    // Set the CS and SS segment selectors
    let (code_selector, data_selector) =
    if context.rip < threads::USER_CODE_START {
        gdt::get_kernel_segments()
    } else {
        gdt::get_user_segments()
    };
    context.cs = code_selector.0 as u64;
    context.ss = data_selector.0 as u64;

    match syscall_id {
        0 => hello_world(),
        1 => sys_write(arg1 as *mut u8, arg2 as usize),
        2 => ipc_write(context_ptr, arg1, arg2),
        3 => ipc_read(context_ptr, arg1),
        4 => sys_yield(context_ptr),
        _ => println!("Unknown syscall {:?} {} {} {}", context_ptr, syscall_id, arg1, arg2)
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
            "mov rax, 0x300",
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
        asm!(
            "mov eax, edx",
            "shr rdx, 32", // Shift high bits into EDX
            "wrmsr",
            in("rcx") MSR_KERNEL_GS_BASE,
            in("rdx") gdt::tss_address()
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
                // println!("ipc_read new_context_addr: {:#x}", new_context_addr);
                cpu::launch_thread(new_context_addr);
            }
        } else {
            // Missing handle
            thread.return_error(SYSCALL_ERROR_INVALID_HANDLE);
            threads::set_current_thread(thread);
        }
    }
}

fn ipc_write(context_ptr: *mut RegisterState, handle: u64, data: u64) {
    // Extract the current thread
    if let Some(mut thread) = threads::take_current_thread() {
        let current_id = thread.id();
        thread.set_context(context_ptr);

        // Get the Rendezvous and call
        if let Some(rdv) = thread.rendezvous(handle) {
            let (thread1, thread2) = rdv.write().send(Some(thread), Message::Short(data));
            // thread1 should be started asap
            // thread2 should be scheduled

            let mut returning = false;
            for maybe_thread in [thread2, thread1] {
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
                // Original thread is waiting.
                // Switch to a different thread
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

fn sys_yield(context_ptr: *mut RegisterState) {
    let next_stack = threads::schedule_next(context_ptr as usize);
    cpu::launch_thread(next_stack);
}

extern "C" fn sys_write(ptr: *mut u8, len: usize) {
    let u8_slice = unsafe {slice::from_raw_parts(ptr, len)};

    if let Ok(s) = str::from_utf8(u8_slice) {
        println!("Write '{}'", s);
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
