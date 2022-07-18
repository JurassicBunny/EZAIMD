//external imports
use clap::Parser;

///Command line arguments to be used by the program
///options must include the Gaussian16 input file.
///The program may be setup such that a simulation can
///restart from a particular time step.
#[derive(Parser, Debug)]
pub struct Args {
    ///input file name
    #[clap(value_name = "INPUT")]
    pub input: String,

    ///time step to be used in fs
    #[clap(short, long, default_value_t = 1.0)]
    pub time_step: f64,

    ///restart simulation from provided step number
    ///requires a simulation save.json to function.
    #[clap(short, long)]
    pub restart: bool,

    ///set the number of steps to be done via
    ///AIMD
    #[clap(short, long, default_value_t = 10000)]
    pub num_steps: usize,

    ///set atoms to be frozen during a simulation
    #[clap(short, long)]
    pub freeze: Option<String>,
}
