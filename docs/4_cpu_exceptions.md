# Handling CPU Exceptions

The CPU can throw a handful of exceptions from page faults to security exceptions. An Interupt Descriptor Table should be set up as a way of handling those exceptions and resuming execution. Each exception has it's own predefined index in the table, this way the hardware can load the exact table entry for each exception type it encounters.

Fortunately a lot of this logic is abstracted away by the `x86_64` crate. This crate utilizes the `x86-interupt` calling convention as a way of using the stack to preserve the register values throughout the exception handling. That way the CPU can pick back up with the instructions using the original register values once the exception has been handled.

This crate doesn't, however, actually handle the exceptions for you. Handler functions still have to be defined, you just don't have to worry about the data structure being used behind the scene to safely get to your handler function. The implementation for this is included in `cpu.rs` and is quite short without all the exception handlers included. Note the use of lazy static again as a way of avoiding unsafe operations around mutable statics and making the compiler happy.

    use x86_64::structures::idt::InterruptDescriptorTable;
    use x86_64::structures::idt::InterruptStackFrame;
    use lazy_static::lazy_static;

    lazy_static! {
        static ref IDT: InterruptDescriptorTable = {
            let mut idt = InterruptDescriptorTable::new();
            idt.breakpoint.set_handler_fn(breakpoint_handler);
            idt
        };
    }

    pub fn init_idt() {
        IDT.load();
    }

    extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
        println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
    }

## Double Faults

Given that a lot of the handlers are unimplemented currently, it makes sense to create what is called the double fault handler. If a handler doesn't exist or is unable to process the error successfully, it results in another exception, which is then processed by the double fault handler. So almost any exceptions that are going to occur at this point are going to end up in the double fault handler since none of the other handlers are implemented. If the double fault handler results in an exception, this is called a triple fault and usually results in a system reset.

    extern "x86-interrupt" fn double_fault_handler(stack_frame: InterruptStackFrame, _error_code: u64) -> ! {
        panic!("EXCEPTION: DOUBLE FAULT\n{:#?}", stack_frame);
    }

More on exception handling can be found [here](https://os.phil-opp.com/cpu-exceptions/) and more on double fault handling can be found [here](https://os.phil-opp.com/double-fault-exceptions/).
