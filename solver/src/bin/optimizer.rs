use clap::Parser;

use solver::problem::*;
use solver::solver_util::volume_optimize;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    input: String,

    #[arg(short, long)]
    output: String,

    #[arg(short, long)]
    solution: String,

    #[arg(long, default_value_t = 10)]
    iteration: i32,

    #[arg(short, long, default_value_t = 0)]
    rand_seed: u128,
}

fn main() {
    let args = Args::parse();
    let input_str = std::fs::read_to_string(args.input).unwrap();
    let input: Input = serde_json::from_str(&input_str).unwrap();

    let solution_str = std::fs::read_to_string(&args.solution).unwrap();
    let original_solution: Solution = serde_json::from_str(&solution_str).unwrap();

    let best_solution = volume_optimize(&input, &original_solution);


    std::fs::write(args.output, serde_json::to_string(&best_solution).unwrap()).unwrap();
}
