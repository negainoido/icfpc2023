use clap::Parser;
use ordered_float::{Float, OrderedFloat};
use pathfinding::kuhn_munkres::kuhn_munkres;
use pathfinding::matrix::Matrix;
use rand::Rng;

use solver::problem::*;
use solver::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    input: String,

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
    let mut rng = rand::thread_rng();
    let seed = args.rand_seed.unwrap_or(rng.gen::<u128>());
    eprintln!("rand seed: {}", seed);
    let mut generator = PlacementGenerator::new(&input, seed);

    let mut best_score = -OrderedFloat::infinity();
    let mut best_placements = vec![];
    let mut iteration_count = 0;

    while get_time() < args.timeout {
        iteration_count += 1;
        let placements = generator.generate();
        assert_eq!(placements.len(), input.musicians.len());

        let mut matrix = Matrix::new(input.musicians.len(), placements.len(), OrderedFloat(0.0));
        let mut reachable_placements = vec![];
        for attendee_id in 0..input.attendees.len() {
            let non_blocked_placement_ids =
                get_non_blocked_placement_ids(input.attendees[attendee_id].pos(), &placements);
            reachable_placements.push(non_blocked_placement_ids);
        }
        for musician_id in 0..input.musicians.len() {
            for attendee_id in 0..input.attendees.len() {
                for &reachable_placement_id in &reachable_placements[attendee_id] {
                    // musician_id を placement_id に対応させたときの attendee_id に対応するスコアを計算
                    let score = input.raw_impact(
                        attendee_id,
                        musician_id,
                        &placements[reachable_placement_id],
                    );
                    matrix[(musician_id, reachable_placement_id)] += score;
                }
            }
        }

        let mut solution: Solution = Default::default();
        solution.placements = placements.clone();
        let (_, assignments) = kuhn_munkres(&matrix);
        let mut new_placements = vec![];
        for assignment in assignments {
            new_placements.push(placements[assignment]);
        }
        solution.placements = new_placements.clone();
        let new_score = solution.score(&input).unwrap();

        if best_score < OrderedFloat(new_score) {
            eprintln!(
                "Improved global score (iteration = {}, input = {}): {} -> {}",
                iteration_count, args.input, best_score.0, new_score
            );
            best_score = OrderedFloat(new_score);
            best_placements = new_placements.clone();
        }
    }

    let mut solution: Solution = Default::default();
    solution.placements = best_placements.clone();
    std::fs::write(args.output, serde_json::to_string(&solution).unwrap()).unwrap();
}
