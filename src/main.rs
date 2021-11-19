extern crate brainmuck_core;

use brainmuck_core::{BrainmuckProgram, CompilationError};
use std::env;
use std::fs;

const SIZE_OF_UNIVERSE: usize = 4096;

fn main() -> Result<(), CompilationError> {
    let args: Vec<_> = env::args().collect();

    if args.len() < 2 {
        println!("brainmuck: you need to provide a program");
        return Ok(());
    }

    let source_text = fs::read(&args[1])?;
    let ast = brainmuck_core::parse(&source_text)?;

    let mut universe = [0u8; SIZE_OF_UNIVERSE];

    let program: Box<dyn BrainmuckProgram> = if should_use_jit(&args) {
        Box::new(brainmuck_core::compile_to_native_code(&ast))
    } else {
        Box::new(brainmuck_core::compile_to_bytecode(&ast))
    };

    program.run(&mut universe);

    Ok(())
}

fn should_use_jit(args: &[String]) -> bool {
    !args[1..].iter().find(|&arg| arg == "--no-jit").is_some()
}
