# User Threads

## User Threads

User thread creation will look a little different than kernel thread creation. Our user thread's will be an entirely separate binary from the kernel, so we'll load them in as an ELF. We'll also create a page table for the user thread. The user thread creation function is quite large so I'll walk through what it does instead of providing all the code here. The code exists in `threads.rs` in the `new_user_thread` function.
- First we have some local imports for the ELF file. These aren't used anywhere else so we might as well just make them local imports
- We then use the ELFBytes crate to read the bytes passed into the function. We verify the headers and read some of the contents from the headers that we need to use such as the instruction pointer.
- We create an empty page table for the new user thread using a function we create in the memory file. I won't go over that function since it's fairly straightforward and commented well.
- Then we switch to the new page table and start copying over the segments the ELF headers ask us to copy over.
- We can create our thread struct now
- We set registers in the context to specify the entrypoint, code and data selectors, set the stack, and pass information to the thread about the heap using registers.
- Our thread is ready to run and can be added to the queue.

## User Space Programs

We have to create a separate Rust program that we can compile in a custom fashion to set some ELF headers telling the kernel where to load our program in memory. A new folder called `user_space` was created where these programs will live (since there will be multiple for testing different functionality). These exist mostly as normal `no_std` Rust programs, but we use some flags related to the linker.

In `.cargo/config.toml` for the new user space program, adding the following tells the linker to specify the code and data location the program should be loaded into.

    [build]
    rustflags = ["-C", "linker-flavor=ld", "-C", "link-args=-Ttext-segment=2000000 -Trodata-segment=2100000", "-C", "relocation-model=static"]

Then after a cargo build we can reference the compiled user space program in our kernel with the following:

    threads::new_user_thread(include_bytes!("../user_space/hello_world/target/target/debug/hello_world"));

There's not much a user program can do right now since it's kind of isolated. At this point it doesn't even have access to write to the VGA buffer. In order to enable us to do something as simple as print hello world, and later make an attempt at IPC, we'll need to implement system calls.