# RL4
An L4 microkernel written in Rust.

## Repository Structure
- `/src` is where the Rust code will live for the microkernel.
- `/docs` contains various documentation and progress reports related to this project.

## Current Objectives
1. Move VGA text buffer code into a separate file and make it safer than it currently is
1. Decide what version of the interface I'll be using (probably X.2) and set up structs for each one of those
1. Implement a handler for the kernel to process system calls
1. Start implementing core system calls in order to be able to do a very basic test of the kernel
