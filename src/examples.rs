//! (only used in test cases)
//! Writes examples to memory

use crate::{ExecutableRegion, WritableRegion};

pub fn write_square_function(buffer: &mut [u8]) {
    let instructions = if cfg!(target_arch = "x86_64") {
        [
            // move rax, rdi
            0x48, 0x89, 0xF8, //
            // imul rax, rdi
            0x48, 0x0F, 0xAF, 0xC7, //
            // ret
            0xC3u8, //
        ]
    } else if cfg!(target_arch = "aarch64") {
        [
            // mul x0, x0, x0
            0x00, 0x7c, 0x00, 0x9b, //
            // ret
            0xc0, 0x03, 0x5f, 0xd6u8,
        ]
    } else {
        panic!("no program for arch")
    };

    let n = instructions.len();
    (&mut buffer[0..n]).copy_from_slice(&instructions);
}

pub fn generate_square_program() -> ExecutableRegion {
    let mut mem = WritableRegion::allocate(4096).unwrap();
    write_square_function(&mut mem[..]);

    mem.into_executable().unwrap()
}
