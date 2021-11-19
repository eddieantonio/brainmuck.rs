extern crate brainmuck_core;
extern crate structopt;

use brainmuck_core::{BrainmuckProgram, CompilationError};
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

const SIZE_OF_UNIVERSE: usize = 4096;

/// Run the program
pub fn run(opt: Opt) -> Result<(), CompilationError> {
    let program = compile_program(&opt)?;

    let mut universe = [0u8; SIZE_OF_UNIVERSE];
    program.run(&mut universe);

    Ok(())
}

fn compile_program(opt: &Opt) -> Result<Box<dyn BrainmuckProgram>, CompilationError> {
    let source_text = fs::read(&opt.program)?;
    let ast = brainmuck_core::parse(&source_text)?;

    if opt.should_use_jit() {
        Ok(Box::new(brainmuck_core::compile_to_native_code(&ast)))
    } else {
        Ok(Box::new(brainmuck_core::compile_to_bytecode(&ast)))
    }
}

/// optizing Brainfuck JIT compiler
///
/// Runs the Brainfuck program by compiling it to machine code.
#[derive(Debug, StructOpt)]
pub struct Opt {
    /// Disable the JIT, using an interpreter instead (slow!)
    #[structopt(short = "-J", long = "--no-jit")]
    no_jit: bool,

    /// filename of the program to run
    #[structopt(name = "PROGRAM")]
    program: PathBuf,
}

impl Opt {
    fn should_use_jit(&self) -> bool {
        !self.no_jit
    }
}
