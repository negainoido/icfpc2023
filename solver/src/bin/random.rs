use clap::Parser;
use geo::Point;
use rand::seq::SliceRandom;
use rand::thread_rng;

use solver::problem::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    input: String,

    #[arg(short, long)]
    output: String,
}

fn main() {
    let args = Args::parse();
    let input_str = std::fs::read_to_string(args.input).unwrap();
    let input: Input = serde_json::from_str(&input_str).unwrap();

    let mut solution: Solution = Default::default();
    let mut cx = input.stage_bottom_left.x() + 10.0;
    let mut cy = input.stage_bottom_left.y() + 10.0;

    for _i in 0..input.musicians.len() {
        solution.placements.push(Point::new(cx, cy));
        cx += 20.0;
        if cx + 10.0 > input.stage_bottom_left.x() + input.stage_width {
            cx = input.stage_bottom_left.x() + 10.0;
            cy += 20.0;
        }
    }

    let mut rnd = thread_rng();
    solution.placements.shuffle(&mut rnd);

    std::fs::write(args.output, serde_json::to_string(&solution).unwrap()).unwrap();
}
