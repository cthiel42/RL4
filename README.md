# RL4
An L4 microkernel written in Rust.

## Repository Structure
- `/src` is where the Rust code will live for the microkernel.
- `/docs` contains various documentation and progress reports related to this project.

## Compiling and Running
To compile the project, run `cargo bootimage` from the root directory. This will generate a file at `target/target/debug/bootimage-rl4.bin` that can then be run on QEMU using the command `qemu-system-x86_64 -drive format=raw,file=target/target/debug/bootimage-rl4.bin`

## Current Objectives
1. IPC
1. Timer for Ping Pong
1. Ping Pong Test
1. Documentation
1. More Documentation
