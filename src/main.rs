#![allow(non_snake_case)]

//program modules
mod atom;
mod cli;
mod simulation;
mod vector;
mod vectored;

//external module
use anyhow::Result;
use clap::Parser;
use simulation::Simulation;

//internal module
use cli::Args;

fn main() -> Result<()> {
    //parse command line arguments
    let args = Args::parse();

    //init a new simulation or restart using the save.json state.
    let simulation = match args.restart {
        true => Simulation::from_save(),
        false => Simulation::new(&args)?.init_forces(),
    };

    simulation.run();

    Ok(())
}
