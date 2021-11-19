extern crate brainmuck_core;
extern crate structopt;

use brainmuck_core::{BrainmuckProgram, CompilationError};
use std::fs;
use std::path::PathBuf;
use structopt::StructOpt;

const SIZE_OF_UNIVERSE: usize = 4096;

fn main() -> Result<(), CompilationError> {
    let opt = Opt::from_args();

    let source_text = fs::read(&opt.program)?;
    let ast = brainmuck_core::parse(&source_text)?;

    let mut universe = [0u8; SIZE_OF_UNIVERSE];

    let program: Box<dyn BrainmuckProgram> = if opt.should_use_jit() {
        Box::new(brainmuck_core::compile_to_native_code(&ast))
    } else {
        Box::new(brainmuck_core::compile_to_bytecode(&ast))
    };

    program.run(&mut universe);

    Ok(())
}

#[derive(Debug, StructOpt)]
struct Opt {
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
