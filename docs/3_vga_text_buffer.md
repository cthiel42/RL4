# Writing to the VGA Text Buffer
In the previous doc, I wrote Hello World to the VGA text buffer in a slightly impractical way. Here I aim to separate out the code for writing to the VGA text buffer to a new file, with the goal of having a safer and more consumable implementation. 

## The VGA Text Buffer
The VGA text buffer is typically a two dimensional array with 25 rows and 80 columns, where each element in the 2D array represents a character on a screen. Each entry is actually 2 bytes, where the first byte represents an ASCII character and the second byte represents how the character should be displayed. That is, what should the background color be, what should the foreground color be, and should the character be blinking. Here I'll be hard coding the color since the output will mostly just be used for debugging purposes, and doesn't require a bunch of colors.

I'll make a new file called `vga` that can then be imported as a module to other parts of the kernel later on. The first bit that I'll add to it is:

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct ScreenChar {
        ascii_character: u8,
        color_code: u8
    }

    const BUFFER_HEIGHT: usize = 25;
    const BUFFER_WIDTH: usize = 80;

    #[repr(transparent)]
    struct Buffer {
        chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
    }

Here `#[derive(Debug, Clone, Copy, PartialEq, Eq)]` is used to make the type printable and comparable. We set the buffer size to 25x80, and then use `[repr(transparent)]` to specify that the Buffer struct should have the exact same layout on memory as it's single value `chars`. To have a way of writing to the screen and keeping track of what is on the screen, we'll make another struct called `Writer`:

    pub struct Writer {
        column_position: usize,
        buffer: &'static mut Buffer,
    }

From there we can implement the writer to write to the buffer, starting with writing a single byte:

    impl Writer {
        pub fn write_byte(&mut self, byte: u8) {
            match byte {
                b'\n' => self.new_line(),
                byte => {
                    if self.column_position >= BUFFER_WIDTH {
                        self.new_line();
                    }

                    let row = BUFFER_HEIGHT - 1;
                    let col = self.column_position;

                    self.buffer.chars[row][col] = ScreenChar {
                        ascii_character: byte,
                        color_code: 15u8,
                    };
                    self.column_position += 1;
                }
            }
        }

        pub fn write_string(&mut self, s: &str) {
            for byte in s.bytes() {
                match byte {
                    // printable ASCII byte or newline
                    0x20..=0x7e | b'\n' => self.write_byte(byte),
                    // not part of printable ASCII range
                    _ => self.write_byte(0xfe),
                }

            }
        }
    }

This pretty much just iterates over the buffer, writing one byte at a time, and then incrementing the counters. Then we can use a function like below to go to a new line when we have a `\n` in the string or we need to wrap our text to the next line. We also want to be able to clear a row in case we run out of rows to write to and need to basically 'scroll' down to another row:

    impl Writer {
        fn new_line(&mut self) {
            for row in 1..BUFFER_HEIGHT {
                for col in 0..BUFFER_WIDTH {
                    let character = self.buffer.chars[row][col].read();
                    self.buffer.chars[row - 1][col].write(character);
                }
            }
            self.clear_row(BUFFER_HEIGHT - 1);
            self.column_position = 0;
        }
        
        fn clear_row(&mut self, row: usize) {
            let blank = ScreenChar {
                ascii_character: b' ',
                color_code: self.color_code,
            };
            for col in 0..BUFFER_WIDTH {
                self.buffer.chars[row][col].write(blank);
            }
        }
    }


We also want to be able to support Rust's formmating too, so we can print various types and use Rust macros like `write!` and `writeln!` with relative ease. We can do that by adding in the following:

    use core::fmt;

    impl fmt::Write for Writer {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            self.write_string(s);
            Ok(())
        }
    }

Now all we need is an interface for using the writer:

    use spin::Mutex;
    use lazy_static::lazy_static;

    lazy_static! {
        pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
            column_position: 0,
            buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
        });
    }

There's a lot of interesting stuff that is going on here and it's all as a result of the unsafe operation we're doing to get a pointer for the buffer. `lazy_static` allows this to be lazy loaded at runtime instead of computing it's value at compilation. We need this since Rust has a lot of problems with the one time initialization of statics that contain non-const functions. We then are using a spin lock to ensure some safety around the mutability of the Writer.

The final piece is to make writing to the buffer a lot easier by creating a macro for `print` and `println`:

    #[macro_export]
    macro_rules! print {
        ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
    }

    #[macro_export]
    macro_rules! println {
        () => ($crate::print!("\n"));
        ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
    }

    #[doc(hidden)]
    pub fn _print(args: fmt::Arguments) {
        use core::fmt::Write;
        WRITER.lock().write_fmt(args).unwrap();
    }

Then within our `main.rs` function we just import this new module with `mod vga;` and can write to the buffer with `print!("Hello World");`
