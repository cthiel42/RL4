# RL4
An L4 microkernel written in Rust.

## Repository Structure
- `/src` is where the Rust code will live for the microkernel.
- `/docs` contains various documentation and progress reports related to this project.

## Compiling and Running
To compile the project, run `cargo bootimage` from the root directory. This will generate a file at `target/target/debug/bootimage-rl4.bin` that can then be run on QEMU using the command `qemu-system-x86_64 -drive format=raw,file=target/target/debug/bootimage-rl4.bin`

## Current Objectives
1. Get a single very simple example of a thread running
1. Decide what version of the interface I'll be using (such as X.2) and set up structs for each one of those
1. Implement a handler for the kernel to process system calls
1. Build out threads to handle scheduling and system calls
