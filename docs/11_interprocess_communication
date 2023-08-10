# Interprocess Communication

## Rendezvous Setup

L4 microkernels use a method of IPC called rendezvous, which basically means both of the threads involved in the message passing are expecting a message to be passed and will block until the message is sent. We'll use an enum in Rust to represent a "handle" that threads can reference to pass a message. This helps with coordination a little bit. When both threads are started, they'll have a handle to this enum that they can reference they both want to use in the IPC system call.

    pub enum Rendezvous {
        Empty,
        Sending(Option<Box<Thread>>, Message),
        Receiving(Box<Thread>)
    }

    pub enum Message {
        Short(u64),
    }

We also create an enum for a message. Right now this is just a single registers worth of data, but the idea is that using an enum like this allows large message sizes to eventually be implemented with only small refactors. These enums exist in a new file called `ipc.rs`. There's also two functions in that file the system calls will use to interact with the rendezvous enums. These mainly handle the state change for the enums in both the read and write case. This is also where the message will be held, so it's also where the message gets set in the register state of the thread it'll be passed back to.

The next step would be the system calls, which are also not too complicated and fairly similar to one another. They both take the current thread and get the correct rendezvous handle based on the register value from the system call. They both then interact with the rendezvous handle using the respective read and write functions in `ipc.rs` to set the correct state of the enum and determine what thread should be called next. This thread then gets returned back to the system call function and launched.

## Ping Pong

From here all we need to do is test our IPC performance with a ping pong test. This will be done by two user space threads reading and writing to one another in a loop. The loop will look like this in the user space program.

    let mut msg: u64 = 0;
    let mut err: u64 = 0;
    loop {
        // send ipc message
        unsafe {
            asm!("mov rax, 2", // write ipc function
                "mov rdi, 0", // First argument
                "syscall",
                in("rsi") msg); // Second argument
        }

        // receive ipc message
        unsafe {
            asm!("mov rax, 3", // read ipc function
                "mov rdi, 0",
                "syscall",
                lateout("rax") err,
                lateout("rdi") msg);
        }

        Print progress
        if msg % 10000 == 0 {
            let mut s = String::<32>::new();
            let _ = write!(s, "ipc read: {msg}");
            unsafe {
                asm!("mov rax, 1", // write syscall function
                    "syscall",
                    in("rdi") s.as_ptr(), // First argument
                    in("rsi") s.len()); // Second argument
            }
        }
        
        msg += 1;
        if msg == 1000000 {
            break;
        }
    }

This will also periodically print our progress out during the test. To ensure both of these threads are started as close to the same time as possible, the new user thread function was modified to return the thread struct instead of automatically queueing it. This way both threads can be created and ready to go, and then the kernel can load them into the queue back to back. Previously this would start the first thread, which would be attempting to run while the kernel thread continued to set up the second thread.

    let thread1 = threads::new_user_thread(include_bytes!("../user_space/ping_pong/target/target/debug/ping_pong"), Vec::from([RENDEZVOUS.clone()]));
    let thread2 = threads::new_user_thread(include_bytes!("../user_space/ping_pong/target/target/debug/ping_pong"), Vec::from([RENDEZVOUS.clone()]));
    println!("Threads created. Adding them to the queue");
    threads::schedule_thread(thread1);
    threads::schedule_thread(thread2);

This also references our rendezvous, which is created as a lazy static in the `main.rs` file.

    lazy_static! {
        static ref RENDEZVOUS: alloc::sync::Arc<RwLock<ipc::Rendezvous>> = alloc::sync::Arc::new(RwLock::new(ipc::Rendezvous::Empty));
    }

Results from this benchmarking test are included in the README.md in the root of this repo.