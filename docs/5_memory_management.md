# Memory Management

## Paging

Our kernel needs to be able to isolate processes from one another. Part of that is ensuring each process has it's own memory to use. Memory Paging is a method where we can break apart our physical memory into smaller chunks, called `pages`, that is then used by a process. A big piece of making this work effectively is using virtual memory addresses to abstract away the fact that the memory is not actually a sequential chunk of physical memory. A translation is done by the kernel to get from the virtual memory to the physical memory. Each page of virtual memory is mapped to the physical memory which is called a `frame`. This mapping between pages and frames needs to be stored somewhere, which is what a `page table` is for. Each process running on the kernel will have it's own page table. A pointer to the currently active page table is stored in it's own CPU register called `CR3`. The kernel needs to load this register with the correct page table pointer for a process before that process is ran, since the CPU will reference this table when doing accessing the memory. 

Multilevel or hierarchical page tables can also be used to cut down on the size of tables that may have large blocks of unallocated memory. The first page table interacted with in this case would have much larger page sizes, and instead of returning frames, it would point to another more fine grained page table that would then return the frame. The x86_64 system uses a 4 level page table and a page size of 4 KiB. Each page table has a fixed size of 512 mappings. These sizing numbers were strategic, as this makes each table fit exactly into one page. Each page table is essentially just an array with 512 entries. A Rust representation would look like this:

    #[repr(align(4096))]
    pub struct PageTable {
        entries: [PageTableEntry; 512],
    }

