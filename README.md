# RL4
An L4 microkernel written in Rust. This is mostly an amalgamation of [blog_os](https://github.com/phil-opp/blog_os) and [EuraliOS](https://github.com/bendudson/EuraliOS), borrowing the relevant aspects and piecing them together. In it's current state, it's functionality is very limited, but built in a way where expanding and adding more system calls would be relatively straightforward. I focused on keeping it simple and implementing the primary objective L4 aims to do well, which is interprocess communication.

# Performance
The standard benchmark test for L4 is a ping pong test. This is where two user threads make IPC calls back and forth to one another. I set up a test to do 1 million iterations of this. Each iteration encompasses both the write from one thread, and the read from another. The test was ran on a 2.6 GHz i7 Intel chip in a MacBook Pro, using QEMU as the hardware emulator. These extra OS and emulator layers no doubt hurt performance, but it lets me get a good ballpark. The average test took a total of 62 seconds, which places a full iteration at about 161,200 clock cycles. The [seL4 Benchmarks](https://sel4.systems/About/Performance/) isolates the read and write performance numbers. For a comparable chip, they have reads taking 638 clock cycles and writes taking 629 clock cycles. This puts a full iteration at 1267 clock cycles, excluding any sort of extra code running for iterating a counter or running an emulator. This makes RL4 at most 127 times slower than seL4.

I suspect some work could be done around the rendezvous implementation and IPC system call handling to reduce the work done during the system calls. I'm also confident running RL4 directly on hardware would yield slightly better performance, but still nowhere near seL4.

## Repository Structure
- `/src` is where the Rust code lives for the microkernel.
- `/docs` contains various documentation and progress reports related to this project. These docs are meant to cover the high level aspects of the kernel and are therefore not comprehensive

## Compiling and Running
To compile the project, run `cargo bootimage` from the root directory. This will generate a file at `target/target/debug/bootimage-rl4.bin` that can then be run on QEMU using the command `qemu-system-x86_64 -drive format=raw,file=target/target/debug/bootimage-rl4.bin`