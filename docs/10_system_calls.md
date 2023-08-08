# System Calls

## System Call Handler

For system calls, there's really not too many components. We have a handler function that has all of our assembly in it, we have our init function that tells our kernel where our handler function is, and then there's a router that the handler function calls in order to call the system call specific functions.

The handler is a lot of assembly but isn't too terribly complicated. Our first bit of assembly switches the stack in use to the kernel. We don't want to be using the user program's stack while doing kernel operations, it kind of breaks a lot of things. We use the GS segment register to do this. While this assembly looks straightforward, there's a lot of details here. EuraliOS docs describe it in far greater detail [here](https://github.com/bendudson/EuraliOS/blob/main/doc/journal/04-more-syscalls.org#switching-stack-on-syscall).

    swapgs
    mov gs:{tss_temp}, rsp
    mov rsp, gs:{tss_timer}
    sub rsp, {ks_offset}
    push gs:{tss_temp}
    swapgs
    push r11
    sub rsp, 8
    push rcx

The next bit of assembly is very similar to the timer interrupt handler. We need to save off all of our registers. Then after that it's time to call our router. Our syscalls are mapped to numbers and specified by the user program using the `rax` register. They then have 3 registers they can use to pass arguments to the system call function. The handler uses the `C` calling convention, hence the reordering of the register values here.

    mov r8, rdx      // Fifth argument <- Syscall third argument
    mov rcx, rsi     // Fourth argument <- Syscall second argument
    mov rdx, rdi     // Third argument <- Syscall first argument
    mov rsi, rax     // Second argument is the syscall number
    mov rdi, rsp     // First argument is the Context address
    call {syscall_router}

Then our system call will go to the router, which is described in the next section, it'll route to the appropriate system call, and then return back to our handler. So then it's time to exit the system call and go back to the user space. We pretty much do everything we just did but in reverse. That is, resetting the stack pointer to the user space and restoring all of our registers.

## System Call Router

The router is what our handler calls as the intermediary between the handler and the actual syscall. The two parts to the function. First we set the code and data selectors depending on whether we're dealing with a kernel thread or a user thread. This ensures we're accessing the correct memory segments while performing the system call. Then we have the actual router, which just matches system call numbers to their respective functions.

    match syscall_id {
        0 => hello_world(),
        1 => sys_write(arg1 as *mut u8, arg2 as usize),
        2 => ipc_write(context_ptr, arg1, arg2),
        3 => ipc_read(context_ptr, arg1),
        4 => sys_yield(context_ptr),
        _ => println!("Unknown syscall {:?} {} {} {}", context_ptr, syscall_id, arg1, arg2)
    }

## System Call Functions

Let's look at one of the functions a little more in depth. `sys_write` is what we'll be using to print text to the VGA buffer from user space programs. It takes in a memory address to a string of text, which it then writes to the buffer. The system call function looks like the following:

    extern "C" fn sys_write(ptr: *mut u8, len: usize) {
        let u8_slice = unsafe {slice::from_raw_parts(ptr, len)};

        if let Ok(s) = str::from_utf8(u8_slice) {
            println!("Write '{}'", s);
        } else {
            println!("Write failed");
        }
    }

## System Calls From User Programs

To actually call the sys_write system call from within a user space program, we need to perform a little bit of assembly. First we create the string we're going to write, then we make our assembly block. Our first line of assembly is setting the `rax` register with the system call number. In our match block in the prior section on our system call router, our sys_write function corresponds to a 1, so that is what we set our register to. Then we set the `rdi` register (our first argument) as the pointer of the string, and the `rsi` register (our second argument) as the length of the string. Since we're not actually passing the string, the system call needs to know at what point to stop trying to write things in memory to the VGA buffer. After all of that, this is what the code in our user program looks like.

    let s = "hello world";
    unsafe {
        asm!("mov rax, 1", // write syscall function
            "syscall",
            in("rdi") s.as_ptr(), // First argument
            in("rsi") s.len()); // Second argument
    }
