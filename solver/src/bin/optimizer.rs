use clap::Parser;
use std::collections::{HashMap, HashSet};

use geo::{EuclideanDistance, Point};
use rand::seq::IteratorRandom;
use rand_pcg::Pcg64Mcg;
use rayon::prelude::*;
use solver::PlacementGenerator;

use solver::problem::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    input: String,

    #[arg(short, long)]
    output: String,

    #[arg(short, long)]
    source: String,

    #[arg(long, default_value_t = 10)]
    iteration: i32,

    #[arg(short, long, default_value_t = 0)]
    rand_seed: u128,
}

fn main() {
    let args = Args::parse();
    let input_str = std::fs::read_to_string(args.input).unwrap();
    let input: Input = serde_json::from_str(&input_str).unwrap();

    let solution_str = std::fs::read_to_string(&args.source).unwrap();
    let original_solution: Solution = serde_json::from_str(&solution_str).unwrap();

    let full = input.pillars.len() > 0;
    let mut best_solution = original_solution.clone();
    let mut best_score = best_solution.score(&input, full).unwrap();

    // Volume optimize
    let mut tmp_solution = best_solution.clone();
    for i in 0..input.musicians.len() {
        let mut current_volume = best_solution
            .volumes
            .clone()
            .unwrap_or(vec![1.0; input.musicians.len()]);

        current_volume[i] = 10.0;
        tmp_solution.volumes = Some(current_volume.clone());
        match tmp_solution.score(&input, full) {
            Ok(score) => {
                if score > best_score {
                    best_score = score;
                    best_solution = tmp_solution.clone();
                    println!("iter {}, score: {}", i, best_score);
                    continue;
                }
            }
            Err(e) => {
                println!("iter {} error exit: {:?}", i, e);
            }
        }
        current_volume[i] = 0.0;
        tmp_solution.volumes = Some(current_volume.clone());
        match tmp_solution.score(&input, full) {
            Ok(score) => {
                if score > best_score {
                    best_score = score;
                    best_solution = tmp_solution.clone();
                    println!("iter {}, score: {}", i, best_score);
                    continue;
                }
            }
            Err(e) => {
                println!("iter {} error exit: {:?}", i, e);
            }
        }
    }

    std::fs::write(args.output, serde_json::to_string(&best_solution).unwrap()).unwrap();
}
