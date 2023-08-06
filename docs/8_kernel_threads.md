# Kernel Threads

## Kernel Threads

The next logical step to implementing threads is to implement kernel threads. These will run using the same page table as the kernel, so outside of switching register values, there isn't much else we need to do to switch between them.

We'll start by creating a file called `threads.rs`. We'll end up adding a lot to this file fairly soon, but for now we can start with a struct for keeping track of threads.

    extern crate alloc;
    use alloc::vec::Vec;
    use spin::RwLock;
    use lazy_static::lazy_static;

    pub struct Thread {
        id: u64,
        kernel_stack: Vec<u8>,
        user_stack: Vec<u8>,
        kernel_stack_end: u64,
        user_stack_end: u64,
        context: u64, // Address of register state on kernel stack
        page_table_physaddr: u64 // Will be used when we get to user threads
    }

    lazy_static! {
        static ref RUNNING_QUEUE: RwLock<VecDeque<Box<Thread>>> = RwLock::new(VecDeque::new());
        static ref CURRENT_THREAD: RwLock<Option<Box<Thread>>> = RwLock::new(None);
        static ref THREAD_COUNTER: RwLock<u64> = RwLock::new(0);
    }

From here we can implement a function that creates a new kernel thread.

    const KERNEL_STACK_SIZE: usize = 4096 * 2;
    const USER_STACK_SIZE: usize = 4096 * 5;
    pub const USER_CODE_START: u64 = 0x2000000;
    pub const USER_CODE_END: u64 = 0x5000000;
    const USER_STACK_START: u64 = 0x3000000;
    const USER_HEAP_START: u64 = 0x280_0060_0000;
    const USER_HEAP_SIZE: u64 = 4 * 1024 * 1024; 

    pub fn new_kernel_thread(function: fn()->()) {
        let new_thread = {
            let kernel_stack = Vec::with_capacity(KERNEL_STACK_SIZE);
            let kernel_stack_end = (VirtAddr::from_ptr(kernel_stack.as_ptr()) + KERNEL_STACK_SIZE).as_u64();
            let user_stack = Vec::with_capacity(USER_STACK_SIZE);
            let user_stack_end = (VirtAddr::from_ptr(user_stack.as_ptr()) + USER_STACK_SIZE).as_u64();
            let context = kernel_stack_end - INTERRUPT_CONTEXT_SIZE as u64;

            Box::new(Thread {
                id: next_id(),
                kernel_stack,
                user_stack,
                kernel_stack_end,
                user_stack_end,
                context,
                page_table_physaddr: 0})
        };

        // Set context registers
        let context = unsafe {&mut *(new_thread.context as *mut RegisterState)};
        context.rip = function as u64;              // Instruction pointer
        context.rsp = new_thread.user_stack_end;    // Stack pointer
        context.rflags = 0x200;                     // Interrupts enabled

        let (code_selector, data_selector) = gdt::get_kernel_segments();
        context.cs = code_selector.0 as u64;
        context.ss = data_selector.0 as u64;

        // Add Thread to RUNNING_QUEUE
        interrupts::without_interrupts(|| {
            RUNNING_QUEUE.write().push_back(new_thread);
        });
    }

There's a few helper functions that have been defined in `gdt.rs` that are referenced within here, and will be referenced within user thread functions as well. This will create a thread and add it to our queue, but we need a way to actual get the thread to run. We can do this by creating a function that can be called by our timer interrupt.

## Scheduling

We can make a function in `threads.rs` that handles the scheduling.

    pub fn schedule_next(context_addr: usize) -> usize {
        let mut running_queue = RUNNING_QUEUE.write();
        let mut current_thread = CURRENT_THREAD.write();

        if let Some(mut thread) = current_thread.take() {
            thread.context = context_addr as u64;
            // thread.page_table_physaddr = get_cr3(); // This will be added in when user threads are implemented
            running_queue.push_back(thread);
        }
        
        // Get the next thread in the queue
        *current_thread = running_queue.pop_front();
        match current_thread.as_ref() {
            Some(thread) => {
                // Set the kernel stack for the next interrupt
                gdt::set_interrupt_stack_table(
                gdt::TIMER_INTERRUPT_INDEX as usize,
                VirtAddr::new(thread.kernel_stack_end));
                // if thread.page_table_physaddr != 0 {      // This will be added in when user threads are implemented
                //     set_cr3(thread.page_table_physaddr);
                // }
                // Point the stack to the new context
                thread.context as usize
            },
            None => 0  // Timer handler won't modify stack
        }
    }

We start by checking if there is a current thread, and storing the context address (which is the register state) before sending it to the back of the queue and attempting to gather the next thread in the queue. The context address then gets returned to the timer interrupt helper. Now if we create two kernel threads our scheduler will switch between them. In `main.rs` we can make two functions:

    fn kernel_thread_main() {
        println!("Kernel thread start");
        threads::new_kernel_thread(kernel_thread_2);
        loop {
            println!("-- 1 --");
            x86_64::instructions::hlt();
        }
    }

    fn kernel_thread_2() {
        loop {
            println!("-- 2 --");
            x86_64::instructions::hlt();
        }
    }

And then in our main function, after we init everything, we can start our first kernel thread with `threads::new_kernel_thread(kernel_thread_main);`. Running the kernel should show switching print statements between one and two.

