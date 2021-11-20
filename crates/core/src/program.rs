//! Defines [BrainmuckProgram] that allows you to run a program, regardless of how it's
//! implemented.

/// Has the same signature as `libc`'s `putchar(3)`.
pub type PutChar = fn(u32) -> u32;
/// Has the same signature as `libc`'s `getchar(3)`.
pub type GetChar = fn() -> u32;

/// A [BrainmuckProgram] is ready to be executed. Just give it some memory!
pub trait BrainmuckProgram {
    /// Run the program with a universe (array of bytes), and a set of IO routines of your
    /// choosing. They must be compatiable with `libc`'s idea of IO.
    fn run_with_custom_io(&self, universe: &mut [u8], putchar: PutChar, getchar: GetChar);

    /// Runs the program with the default IO (prints to `stdout`; accepts input from `stdin`)
    fn run(&self, universe: &mut [u8]) {
        self.run_with_custom_io(universe, putchar, getchar);
    }
}

/// Emulates libc's `putchar(3)`
fn putchar(c: u32) -> u32 {
    print!("{}", (c & 0xFF) as u8 as char);
    1
}

/// Emulates libc's `getchar(3)`
fn getchar() -> u32 {
    use std::io::{self, Read};
    let mut one_byte = [0u8];
    io::stdin()
        .read_exact(&mut one_byte)
        .expect("could not read even a single byte!");
    one_byte[0] as u32
}
