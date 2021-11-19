use std::error::Error;

use structopt::StructOpt;

use brainmuck::{run, Opt};

fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opt::from_args();

    run(opt)
}
