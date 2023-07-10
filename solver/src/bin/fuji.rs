use clap::Parser;
use geo::Point;
use rand::Rng;
use solver::get_time;

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

    #[arg(short = 'R', long)]
    rand_seed: Option<u128>,

    #[arg(short, long)]
    reduced_attendee: Option<usize>,
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

    let volumes = solution.volumes.clone();

    let placements = fuji(
        &input,
        &mut solution.placements,
        &solution
            .volumes
            .unwrap_or(vec![10.0; input.musicians.len()]),
        args.timeout,
        seed,
        args.reduced_attendee.unwrap_or(input.attendees.len()),
    );

    let solution = Solution {
        placements,
        volumes,
    };
    let solution = volume_optimize(&input, &solution);

    std::fs::write(args.output, serde_json::to_string(&solution).unwrap()).unwrap()
}

fn fuji(
    input: &Input,
    best: &mut Vec<Point>,
    best_volume: &[f64],
    timeout: f64,
    rand_seed: u128,
    reduce_num: usize,
) -> Vec<Point> {
    let mut rng = rand_pcg::Pcg64Mcg::new(rand_seed);
    let input = reduce_attendees(input, reduce_num);
    let mut best_score = input
        .score_fast(&Solution {
            placements: best.clone(),
            volumes: Some(best_volume.to_owned()),
        })
        .unwrap();

    let mut current = best.clone();
    let mut temp: f64 = 1.0;
    let temp_min: f64 = 0.00001;
    let alpha = 0.9;

    while get_time() < timeout {
        let idx = rng.gen_range(0..best.len());
        let dir = rng.gen_range(0..4);
        let dx = [0.0, 1.0, 0.0, -1.0];
        let dy = [1.0, 0.0, -1.0, 0.0];
        let step = rng.gen_range(1..100) as f64;
        let x_old = current[idx].x();
        let y_old = current[idx].y();
        current[idx].set_x(x_old + dx[dir] * step);
        current[idx].set_y(y_old + dy[dir] * step);

        if input.is_valid_placements(&current).is_err() {
            current[idx].set_x(x_old);
            current[idx].set_y(y_old);
            continue;
        }
        let solution = Solution {
            placements: current.to_owned(),
            volumes: Some(best_volume.to_owned()),
        };

        let current_score = input.score_fast(&solution);
        if let Ok(sc) = current_score {
            if sc > best_score {
                eprintln!(
                    "score for reduced attendees is improved: {} -> {}",
                    best_score, sc,
                );
                best_score = sc;
                *best = solution.placements;
            } else if ((sc - best_score) / temp).exp() > rng.gen::<f64>() {
                eprintln!("yaku(temp={}): {} -> {}", temp, best_score, sc);
            } else {
                current[idx].set_x(x_old);
                current[idx].set_y(y_old);
            }
        }
        temp = temp_min.max(temp * alpha);
    }
    best.to_vec()
}
