## Dependencies
- [Rust](https://www.rust-lang.org/tools/install) - Obviously you need this. Rust's website has simple installation instructions 
- [QEMU](https://www.qemu.org/download/) - Not a dependency per se but this makes testing the kernel locally really easy. It's included in most package managers. For Ubuntu it's `sudo apt-get install qemu-kvm`

## Helpful Rust Resources
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/) - This is a really good primer on developing in Rust. Spend a few hours to read through this to understand it's syntax and how it behaves. This site houses all of Rust's documentation, so the next logical step would be to look through what else they have.
- [OS in Rust](https://os.phil-opp.com/) - This is a fairly robust demo of running Rust on metal and building an OS out of it
- [Bare Metal Rust](https://google.github.io/comprehensive-rust/bare-metal.html) - An additional resource for how to run Rust on bare metal.
- [EuraliOS Docs](https://github.com/bendudson/EuraliOS/tree/main/doc/journal) - Picks up where 'OS in Rust' leaves off

## Helpful L4 Resources
- [seL4 Whitepaper](https://sel4.systems/About/seL4-whitepaper.pdf) - An introduction of what the seL4 microkernel is and isn't, seL4 verification, a history of L4 microkernels, and L4 use cases.
- [seL4 GitHub](https://github.com/seL4/seL4)
- [seL4 Tutorials](https://docs.sel4.systems/Tutorials/)
- [L4Ka Pistachio](https://github.com/l4ka/pistachio/)
- [A Functional Approach to Memory-Safe Operating Systems](https://pdxscholar.library.pdx.edu/cgi/viewcontent.cgi?article=1498&context=open_access_etds) - An Implementation of an L4 microkernel in Haskell
- [L4 Version X.2 Interface](http://www.cse.unsw.edu.au/~cs9242/05/project/l4-x2.pdf) - The X.2 interface reference manual, also commonly referred to as Version 4 of the L4 interface
- [Thread Context Switching](https://samwho.dev/blog/context-switching-on-x86/)
