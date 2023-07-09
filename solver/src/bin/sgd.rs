use std::ops::Mul;

use clap::Parser;
use geo::{EuclideanDistance, Point};
use rand::{seq::SliceRandom, Rng};

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

    #[arg(long, default_value_t = 1197)]
    rand_seed: u128,
}

fn main() {
    let args = Args::parse();
    let input_str = std::fs::read_to_string(args.input).unwrap();
    let input: Input = serde_json::from_str(&input_str).unwrap();

    let mut solution: Solution = Default::default();
    let leftlimit = input.stage_bottom_left.x() + 10.0;
    let rightlimit = input.stage_bottom_left.x() + input.stage_width - 10.0;
    let bottomlimit = input.stage_bottom_left.y();
    let toplimit = input.stage_bottom_left.y() + input.stage_height - 10.0;

    let mut rng = rand::thread_rng();
    for _i in 0..input.musicians.len() {
        let x = rng.gen_range(leftlimit..rightlimit);
        let y = rng.gen_range(bottomlimit..toplimit);
        solution.placements.push(Point::new(x, y));
    }

    let mut best_solution = solution.clone();
    let mut best_score = solution.score(&input, false).unwrap();
    println!("initial score: {}", best_score);

    let mut permutation = vec![];
    for i in 0..input.attendees.len() {
        for k in 0..input.musicians.len() {
            permutation.push((i, k));
        }
    }

    let rate = 0.1;
    for i in 0..args.iteration {
        permutation.shuffle(&mut rng);
        for (i, k) in &permutation {
            let a = &input.attendees[*i];
            let a = Point::new(a.x, a.y);
            let d = solution.placements[*k].euclidean_distance(&a);
            let diff = solution.placements[*k] - a;
            let g = diff.mul(-(2.0 / (d * d * d * d)) * rate);
            solution.placements[*k] += g;
        }

        let score = solution.score(&input, false);
        println!("iter {}, score: {:?}", i, score);
        match score {
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
