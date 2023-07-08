use clap::Parser;
use geo::Point;
use ordered_float::OrderedFloat;
use pathfinding::kuhn_munkres::kuhn_munkres;
use pathfinding::matrix::Matrix;

use solver::problem::*;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    input: String,

    #[arg(short, long)]
    output: String,

    #[arg(short, long, default_value_t = 60.0)]
    timeout: f64,
}

struct PlacementGenerator {
    input: Input,
    cartesian_coordinate_candidates: Vec<Point>,
    honeycomb_candidates: Vec<Point>,
}

impl PlacementGenerator {
    fn new(input: Input) -> Self {
        let cartesian_coordinate_candidates =
            PlacementGenerator::cartesian_coordinate_candidates(&input);

        let honeycomb_candidates = PlacementGenerator::honeycomb_candidates(&input);

        PlacementGenerator {
            input,
            cartesian_coordinate_candidates,
            honeycomb_candidates,
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
        if self.cartesian_coordinate_candidates.len() < self.input.musicians.len() {
            self.honeycomb_candidates
                .iter()
                .take(self.input.musicians.len())
                .cloned()
                .collect()
        } else {
            self.cartesian_coordinate_candidates
                .iter()
                .take(self.input.musicians.len())
                .cloned()
                .collect()
        }
    }
}

fn main() {
    let args = Args::parse();
    let input_str = std::fs::read_to_string(args.input).unwrap();
    let input: Input = serde_json::from_str(&input_str).unwrap();
    let mut generator = PlacementGenerator::new(input.clone());
    let placements = generator.generate();
    assert_eq!(placements.len(), input.musicians.len());

    let mut matrix = Matrix::new(input.musicians.len(), placements.len(), OrderedFloat(0.0));

    let mut reachable_placements = vec![];
    let mut score = 0.0;
    for attendee_id in 0..input.attendees.len() {
        let detail = input.score_attendee_fast(attendee_id, &placements);
        reachable_placements.push(detail.matched_musician_ids);
        score += detail.score;
    }
    dbg!(score);
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

    let mut debug_score = OrderedFloat(0.0);
    for musician_id in 0..input.musicians.len() {
        debug_score += matrix[(musician_id, musician_id)];
    }
    dbg!(debug_score);

    let mut solution: Solution = Default::default();
    solution.placements = placements.clone();
    dbg!(solution.score(&input));
    let (score, assignments) = kuhn_munkres(&matrix);
    dbg!(score);

    let mut new_placements = vec![];
    for assignment in assignments {
        new_placements.push(placements[assignment].clone());
    }
    // let mut new_placements = vec![Point::default(); placements.len()];
    // for i in 0..assignments.len() {
    //     new_placements[assignments[i]] = placements[i].clone();
    //     // new_placements.push(placements[assignment].clone());
    // }
    solution.placements = new_placements;
    dbg!(solution.score(&input));
    std::fs::write(args.output, serde_json::to_string(&solution).unwrap()).unwrap();
}
