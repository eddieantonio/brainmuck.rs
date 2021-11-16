extern crate brainmuck;

use brainmuck::CompilationError;
use std::env;
use std::fs;

const SIZE_OF_UNIVERSE: usize = 4096;

fn main() -> Result<(), CompilationError> {
    let args: Vec<_> = env::args().collect();

    if args.len() < 2 {
        println!("brainmuck: you need to provide a program");
        return Ok(());
    }

    // HACK: for some bizare reason, source_text was being double free'd...
    // Or... something ¯\_(ツ)_/¯
    let ast = {
        let source_text = fs::read(&args[1])?;
        brainmuck::parse(&source_text)?
    };

    let mut universe = [0u8; SIZE_OF_UNIVERSE];
    if should_use_jit(&args) {
        let program = brainmuck::jit_compile(&ast);
        program.run(&mut universe);
    } else {
        let program = brainmuck::compile_to_bytecode(&ast);
        brainmuck::bytecode::interpret(&program, &mut universe);
    }

    Ok(())
}

fn should_use_jit(args: &[String]) -> bool {
    !args[1..].iter().find(|&arg| arg == "--no-jit").is_some()
}
