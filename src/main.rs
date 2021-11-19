use std::process;

use structopt::StructOpt;

use brainmuck::{run, Opt};

fn main() {
    let opt = Opt::from_args();

    if let Err(err) = run(opt) {
        eprintln!("{}", err);
        process::exit(1);
    }
}
