use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;

fn main() {
    /* // Uncomment this to generate the elf_data.rs file
    let input_file = Path::new("user_space/hello_world/target/target/debug/hello_world");
    let output_file = Path::new("elf_data.rs");

    let mut file = File::open(input_file).expect("Failed to open input file");
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).expect("Failed to read input file");

    let mut output_file = File::create(output_file).expect("Failed to create output file");
    output_file.write_all(b"pub const ELF_DATA: &[u8] = &[")
        .expect("Failed to write output file");
    for (i, byte) in contents.iter().enumerate() {
        if i > 0 {
            output_file.write_all(b", ").expect("Failed to write output file");
        }
        output_file.write_all(format!("0x{:02X}", byte).as_bytes())
            .expect("Failed to write output file");
    }
    output_file.write_all(b"];").expect("Failed to write output file");
    */
}

