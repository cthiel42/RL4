# Timer Interrupts

## Timer Interrupt

As a start to implementing system calls, threads, and context switching between those threads, we can start by adding a timer interrupt. This will periodically interrupt the running thread and switch to another if needed in order to make sure one thread doesn't hog all the CPU time. This will also be the start of storing register states, so we define a struct in `arch/arch.rs` to store register values.

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

Now we're able to create our timer interrupt within the `cpu.rs` file along with the other fault handlers.

    use crate::arch::arch::RegisterState;

    extern "C" fn timer_interrupt_helper(context: &mut RegisterState) -> usize {
        let next_stack = context.rsp; // This will eventually call a scheduler function to find the stack of the next thread to run
        unsafe {
            PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
        }
        next_stack
    }

    // Add this line to the IDT init
    idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler).set_stack_index(gdt::TIMER_INTERRUPT_INDEX);

The actual interrupt handler is quite a bit of assembly, and is somewhat well commented, so I haven't placed it in these docs. Most of the code is borrowed from EuraliOS. The high level overview of it is that it pushes all the register values onto the stack, calls our interrupt helper function above, and then pops all the register values from the stack.