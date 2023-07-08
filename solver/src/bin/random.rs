use clap::Parser;
use geo::Point;
use rand::seq::SliceRandom;
use rand_pcg::Pcg64Mcg;

use solver::problem::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    input: String,

    #[arg(short, long)]
    output: String,

    #[arg(long, default_value_t = 10)]
    iteration: i32,

    #[arg(short, long)]
    timeout: Option<i32>,

    #[arg(short, long, default_value_t = 0)]
    rand_seed: u128,
}

fn main() {
    let args = Args::parse();
    let input_str = std::fs::read_to_string(args.input).unwrap();
    let input: Input = serde_json::from_str(&input_str).unwrap();

    let mut solution: Solution = Default::default();
    let musician_dist = 10.0 + 1e-7;
    let mut cx = input.stage_bottom_left.x() + musician_dist;
    let mut cy = input.stage_bottom_left.y() + musician_dist;
    let mut j = 0;
    for _i in 0..input.musicians.len() {
        solution.placements.push(Point::new(cx, cy));
        cx += musician_dist;
        if cx + musician_dist > input.stage_bottom_left.x() + input.stage_width {
            cx = input.stage_bottom_left.x() + musician_dist;
            j += 1;
            if j % 2 == 1 {
                cx += musician_dist / 2.0;
            }
            cy += musician_dist * f64::sqrt(3.0) / 2.0;
        }
    }

    let mut rnd = Pcg64Mcg::new(args.rand_seed);
    let mut best_solution = solution.clone();
    let mut best_score = solution.score(&input).unwrap();
    println!("initial score: {}", best_score);
    for i in 0..args.iteration {
        solution.placements.shuffle(&mut rnd);

        match solution.score(&input) {
            Ok(score) => {
                if score > best_score {
                    best_score = score;
                    best_solution = solution.clone();
                    println!("iter {}, score: {}", i, best_score);
                }
            }
            Err(_) => {}
        }
    }

    std::fs::write(args.output, serde_json::to_string(&best_solution).unwrap()).unwrap();
}
