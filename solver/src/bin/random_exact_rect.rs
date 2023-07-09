use clap::Parser;
use geo::Point;
use ordered_float::OrderedFloat;
use pathfinding::kuhn_munkres::kuhn_munkres;
use pathfinding::matrix::Matrix;

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

    #[arg(short, long, default_value_t = 0)]
    rand_seed: u128,
}

fn generate_first_level_candidates(input: &Input) -> Vec<Point> {
    let x_count = (input.stage_width / 10.0).floor() as usize - 1;
    let y_count = (input.stage_height / 10.0).floor() as usize - 1;
    dbg!(x_count, y_count);

    let candidates = solver::PlacementGenerator::honeycomb_candidates(input);

    // Generate first level candidate
    let x_gap = if x_count > 1 {
        (input.stage_width - 20.0) / (x_count - 1) as f64
    } else {
        0.0
    };
    let y_gap = if y_count > 1 {
        (input.stage_height - 20.0) / (y_count - 1) as f64
    } else {
        0.0
    };

    // Prefer candidates closer to the stage borders
    let mut layered_candidates = vec![];

    for candidate in &candidates {
        let x_level = ((candidate.x() - input.stage_bottom_left.x())
            .min(input.stage_bottom_left.x() + input.stage_width - candidate.x())
            .floor()
            / 7.0) as usize;
        let y_level = ((candidate.y() - input.stage_bottom_left.y())
            .min(input.stage_bottom_left.y() + input.stage_height - candidate.y())
            .floor()
            / 7.0) as usize;
        let level = x_level.min(y_level);
        while level + 1 > layered_candidates.len() {
            layered_candidates.push(vec![]);
        }
        layered_candidates[level].push(candidate);
    }

    /*     for i in 0..x_count {
           for j in 0..y_count {
               let offset = Point::new(10.0 + i as f64 * x_gap, 10.0 + j as f64 * y_gap);
               let pos = input.stage_bottom_left + offset;
               let x_level = i.min(x_count - 1 - i);
               let y_level = j.min(y_count - 1 - j);
               let level = x_level.min(y_level);
               while level + 1 > layered_candidates.len() {
                   layered_candidates.push(vec![]);
               }
               layered_candidates[level].push(pos);
           }
       }
    */
    let mut candidates = vec![];
    for level in 0..layered_candidates.len() {
        candidates.extend(layered_candidates[level].clone());
        if candidates.len() >= input.musicians.len() * 2 {
            break;
        }
    }
    candidates
}

fn exact_match_candidates(input: &Input, candidates: &Vec<Point>) -> Vec<Point> {
    let mut matrix = Matrix::new(input.musicians.len(), candidates.len(), OrderedFloat(0.0));
    let mut reachable_candidates = vec![];
    for attendee_id in 0..input.attendees.len() {
        let non_blocked_candidate_ids =
            get_non_blocked_placement_ids(input.attendees[attendee_id].pos(), &candidates);
        reachable_candidates.push(non_blocked_candidate_ids);
    }

    for musician_id in 0..input.musicians.len() {
        for attendee_id in 0..input.attendees.len() {
            for &reachable_candidate_id in &reachable_candidates[attendee_id] {
                // musician_id を placement_id に対応させたときの attendee_id に対応するスコアを計算
                let score = input.raw_impact(
                    attendee_id,
                    musician_id,
                    &candidates[reachable_candidate_id],
                );
                matrix[(musician_id, reachable_candidate_id)] += score;
            }
        }
    }
    let (_, assignments) = kuhn_munkres(&matrix);
    let mut filtered_candidates = vec![];
    for assignment in assignments {
        filtered_candidates.push(candidates[assignment].clone());
    }
    filtered_candidates
}

fn solve(input: &Input) -> Vec<Point> {
    let first_level_candidates = generate_first_level_candidates(input);
    // 最初はmusicianよりも多い候補地を用いて最適化を行い、その結果に基づき二段階目の最適化に使用する候補地を列挙
    let second_level_candidates = exact_match_candidates(input, &first_level_candidates);
    exact_match_candidates(input, &second_level_candidates)
}

fn main() {
    let args = Args::parse();
    let input_str = std::fs::read_to_string(args.input.clone()).unwrap();
    let input: Input = serde_json::from_str(&input_str).unwrap();

    let x_count = (input.stage_width / 10.0).floor() as usize - 1;
    let y_count = (input.stage_height / 10.0).floor() as usize - 1;
    let best_placements = if x_count * y_count >= input.musicians.len() {
        // Use new strategy!
        solve(&input)
    } else {
        // Give up
        let mut generator = PlacementGenerator::new(input.clone(), args.rand_seed);
        generator.generate()
    };
    let mut solution: Solution = Default::default();
    solution.placements = best_placements.clone();
    input.is_valid_placements(&best_placements).unwrap();
    eprintln!("Solver score: {}", solution.score(&input).unwrap());
    std::fs::write(args.output, serde_json::to_string(&solution).unwrap()).unwrap();
}
