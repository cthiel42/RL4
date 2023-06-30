pub fn copy_memory(destination: usize, source: &[u8]) {
    let mut page_ptr: *mut u8 = destination as *mut u8;
    for i in 0..source.len() {
        unsafe { page_ptr.write_volatile(source[i]) };
        unsafe { page_ptr = page_ptr.offset(1) };
    }
}