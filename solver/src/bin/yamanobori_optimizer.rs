use clap::Parser;
use rand::Rng;

use solver::problem::*;
use solver::solver_util::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    input: String,

    #[arg(short, long)]
    solution: String,

    #[arg(short, long)]
    output: String,

    #[arg(short, long, default_value_t = 30.0)]
    timeout: f64,

    #[arg(short, long)]
    rand_seed: Option<u128>,
}

fn main() {
    let args = Args::parse();
    let input_str = std::fs::read_to_string(args.input.clone()).unwrap();
    let input: Input = serde_json::from_str(&input_str).unwrap();

    let solution_str = std::fs::read_to_string(args.solution.clone()).unwrap();
    let mut solution: Solution = serde_json::from_str(&solution_str).unwrap();

    let mut rng = rand::thread_rng();

    let seed = args.rand_seed.unwrap_or(rng.gen::<u128>());
    eprintln!("rand seed: {}", seed);

    let mut best_score = input.score_fast(&solution).unwrap();
    let volumes = solution.volumes.clone();

    let placements = yamanobori(
        &input,
        &mut best_score,
        &mut solution.placements,
        &solution
            .volumes
            .unwrap_or(vec![10.0; input.musicians.len()]),
        args.timeout,
    );

    let solution = Solution {
        placements,
        volumes,
    };
    let solution = volume_optimize(&input, &solution);

    std::fs::write(args.output, serde_json::to_string(&solution).unwrap()).unwrap()
}
