use brainmuck::{run, Opt};
use brainmuck_core::CompilationError;
use structopt::StructOpt;

fn main() -> Result<(), CompilationError> {
    let opt = Opt::from_args();

    run(opt)
}
