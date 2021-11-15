extern crate brainmuck;

use brainmuck::CompilationError;
use std::env;
use std::fs;

const SIZE_OF_UNIVERSE: usize = 4096;

fn main() -> Result<(), CompilationError> {
    let args: Vec<_> = env::args().collect();

    if args.len() != 2 {
        println!("usage error: need exactly one argument");
        return Ok(());
    }

    let source_text = fs::read(&args[1])?;
    let ast = brainmuck::parse(&source_text)?;
    let program = brainmuck::compile_to_bytecode(&ast);

    let mut universe = [0u8; SIZE_OF_UNIVERSE];

    brainmuck::run_native_code(&ast);

    brainmuck::bytecode::interpret(&program, &mut universe);

    Ok(())
}
