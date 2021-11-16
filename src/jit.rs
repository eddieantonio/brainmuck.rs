use mmap_jit::{as_function, ExecutableRegion, WritableRegion};

pub struct CompiledProgram {
    code: ExecutableRegion,
}

type Program = fn(*mut u8, fn(u32) -> u32, fn() -> u32) -> u64;

impl CompiledProgram {
    pub fn from_binary(binary: &[u8]) -> CompiledProgram {
        let mut mem = WritableRegion::allocate(binary.len()).unwrap();
        (&mut mem[0..binary.len()]).copy_from_slice(&binary);

        CompiledProgram {
            code: mem.into_executable().unwrap(),
        }
    }

    pub fn run(&self, universe: &mut [u8]) {
        let program = unsafe { as_function!(self.code, Program) };

        program(universe.as_mut_ptr(), putchar, getchar);
    }
}

fn getchar() -> u32 {
    use std::io::{self, Read};
    let mut one_byte = [0u8];
    io::stdin()
        .read_exact(&mut one_byte)
        .expect("could not read even a single byte!");
    one_byte[0] as u32
}

fn putchar(c: u32) -> u32 {
    print!("{}", (c & 0xFF) as u8 as char);
    1
}
