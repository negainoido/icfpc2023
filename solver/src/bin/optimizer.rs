use std::collections::HashMap;
use std::mem;
use std::time::Duration;
use clap::Parser;
use rand::seq::IteratorRandom;
use rand_pcg::Pcg64Mcg;
use rayon::prelude::*;
use solver::PlacementGenerator;
use rand::prelude::SliceRandom;

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
    let full = input.pillars.len() > 0;
    let mut rnd = Pcg64Mcg::new(args.rand_seed);

    let mut instruments = HashMap::new();
    for &m in &input.musicians {
        let count = instruments.get(&m).unwrap_or(&0);
        instruments.insert(m, count + 1);
    }
    let mut musician_map = vec![Vec::new(); instruments.keys().len()];
    for (i, &m) in input.musicians.iter().enumerate() {
        musician_map[m].push(i);
    }
    let musician_map = musician_map;
    if musician_map.len() < 2 {
        panic!("musicians are too few");
    }
    println!("musicians: {:?}", musician_map);
    let mut best_solution = volume_optimize(&input, &original_solution);
    let mut best_score = input.score_fast(&best_solution, full).unwrap();
    let now = std::time::Instant::now();

    while now.elapsed() < Duration::from_secs(30) {
        println!("trying");
        let mut new_solution = best_solution.clone();
        let target_insts = (0..musician_map.len()).choose_multiple(&mut rnd, 2);
        let target_insts = (target_insts[0], target_insts[1]);
        let left = *musician_map[target_insts.0].choose(&mut rnd).unwrap();
        let right = *musician_map[target_insts.1].choose(&mut rnd).unwrap();
        let tmp = new_solution.placements[left];
        new_solution.placements[left] = new_solution.placements[right];
        new_solution.placements[right] = tmp;
        let mut flag = false;
        match input.score_fast(&new_solution, full) {
            Ok(new_score) => {
                if new_score > best_score {
                    best_solution = new_solution.clone();
                    best_score = new_score;
                    flag = true;
                }
            }
            Err(_) => {
                println!("invalid solution");
            }
        }
        if flag {
            println!("best score: {}", best_score);
        }
    }

    let best_solution = volume_optimize(&input, &original_solution);

    std::fs::write(args.output, serde_json::to_string(&best_solution).unwrap()).unwrap();
}
