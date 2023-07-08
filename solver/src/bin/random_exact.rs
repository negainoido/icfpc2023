use clap::Parser;
use geo::Point;
use ordered_float::{Float, OrderedFloat};
use pathfinding::kuhn_munkres::kuhn_munkres;
use pathfinding::matrix::Matrix;
use rand::prelude::SliceRandom;
use rand::Rng;
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

    #[arg(short, long, default_value_t = 30.0)]
    timeout: f64,

    #[arg(short, long, default_value_t = 0)]
    rand_seed: u128,
}

fn get_time() -> f64 {
    static mut STIME: f64 = -1.0;
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    let ms = t.as_secs() as f64 + t.subsec_nanos() as f64 * 1e-9;
    unsafe {
        if STIME < 0.0 {
            STIME = ms;
        }
        ms - STIME
    }
}

struct PlacementGenerator {
    input: Input,
    cartesian_coordinate_candidates: Vec<Point>,
    honeycomb_candidates: Vec<Point>,
    rand_gen: Pcg64Mcg,
}

impl PlacementGenerator {
    fn new(input: Input, rand_seed: u128) -> Self {
        let rand_gen = Pcg64Mcg::new(rand_seed);
        let cartesian_coordinate_candidates =
            PlacementGenerator::cartesian_coordinate_candidates(&input);

        let honeycomb_candidates = PlacementGenerator::honeycomb_candidates(&input);

        PlacementGenerator {
            input,
            cartesian_coordinate_candidates,
            honeycomb_candidates,
            rand_gen: rand_gen,
        }
    }

    fn cartesian_coordinate_candidates(input: &Input) -> Vec<Point> {
        let mut candidates = vec![];
        let mut cx = input.stage_bottom_left.x() + 10.0;
        let mut cy = input.stage_bottom_left.y() + 10.0;
        loop {
            candidates.push(Point::new(cx, cy));
            cx += 20.0;
            if cx + 10.0 > input.stage_bottom_left.x() + input.stage_width {
                cx = input.stage_bottom_left.x() + 10.0;
                cy += 20.0;
            }
            if cy + 10.0 > input.stage_bottom_left.y() + input.stage_height {
                break;
            }
        }
        candidates
    }

    fn honeycomb_candidates(input: &Input) -> Vec<Point> {
        let mut candidates = vec![];
        let musician_dist = 10.0;
        let mut cx = input.stage_bottom_left.x() + musician_dist;
        let mut cy = input.stage_bottom_left.y() + musician_dist;
        let mut j = 0;
        loop {
            candidates.push(Point::new(cx, cy));
            cx += musician_dist;
            if cx + musician_dist > input.stage_bottom_left.x() + input.stage_width {
                cx = input.stage_bottom_left.x() + musician_dist;
                j += 1;
                if j % 2 == 1 {
                    cx += musician_dist / 2.0;
                }
                cy += musician_dist * f64::sqrt(3.0 + 1e-7) / 2.0;
            }

            if cy + musician_dist > input.stage_bottom_left.y() + input.stage_height {
                break;
            }
        }
        candidates
    }

    fn generate(&mut self) -> Vec<Point> {
        // self.rand_gen.gen_range()
        let mut candidates =
            if self.cartesian_coordinate_candidates.len() < self.input.musicians.len() {
                self.honeycomb_candidates.clone()
            } else if self.honeycomb_candidates.len() < self.input.musicians.len() {
                self.cartesian_coordinate_candidates.clone()
            } else {
                if self.rand_gen.gen_bool(0.5) {
                    self.honeycomb_candidates.clone()
                } else {
                    self.cartesian_coordinate_candidates.clone()
                }
            };
        candidates.shuffle(&mut self.rand_gen);
        candidates
            .iter()
            .take(self.input.musicians.len())
            .cloned()
            .collect()
    }
}

fn main() {
    let args = Args::parse();
    let input_str = std::fs::read_to_string(args.input.clone()).unwrap();
    let input: Input = serde_json::from_str(&input_str).unwrap();
    let mut generator = PlacementGenerator::new(input.clone(), args.rand_seed);

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
            let detail = input.score_attendee_fast(attendee_id, &placements);
            reachable_placements.push(detail.matched_musician_ids);
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
        let original_score = solution.score(&input).unwrap();
        let (_, assignments) = kuhn_munkres(&matrix);
        let mut new_placements = vec![];
        for assignment in assignments {
            new_placements.push(placements[assignment].clone());
        }
        solution.placements = new_placements.clone();
        let new_score = solution.score(&input).unwrap();
        eprintln!(
            "Placement original score (iteration = {}, input = {}): {}",
            iteration_count, args.input, original_score
        );
        eprintln!(
            "Placement optimized score (iteration = {}, input = {}): {}",
            iteration_count,
            args.input,
            solution.score(&input).unwrap()
        );

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
