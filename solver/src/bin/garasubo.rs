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

    #[arg(long, default_value_t = 5)]
    iteration: i32,

    #[arg(short, long, default_value_t = 0)]
    rand_seed: u128,
}

const PICK_POINTS_COUNT: usize = 50;

fn main() {
    let args = Args::parse();
    let input_str = std::fs::read_to_string(args.input).unwrap();
    let input: Input = serde_json::from_str(&input_str).unwrap();

    let mut solution: Solution = Default::default();

    let mut instruments = HashMap::new();
    for &m in &input.musicians {
        let count = instruments.get(&m).unwrap_or(&0);
        instruments.insert(m, count + 1);
    }
    let mut musician_map = vec![Vec::new(); instruments.keys().len()];
    for (i, &m) in input.musicians.iter().enumerate() {
        musician_map[m].push(i);
    }
    println!("musicians: {:?}", musician_map);
    // musiciansが多すぎるとかの場合はそんなにいいスコアでないので諦める
    if musician_map.len() >= 30 {
        println!("too many musicians or no pillars");
        return;
    }
    let full_div = input.pillars.len() > 0;

    // 各楽器のそれっぽい人気度を計算
    let mut popularity = Vec::with_capacity(instruments.keys().len());
    let stage_center_x = input.stage_bottom_left.x() + input.stage_width / 2.0;
    let stage_center_y = input.stage_bottom_left.y() + input.stage_height / 2.0;
    let stage_center = Point::new(stage_center_x, stage_center_y);
    let visible_attendees = input.get_visible_attendees(stage_center, &vec![]);
    for i in 0..instruments.keys().len() {
        popularity.push((
            input.raw_score_for_instrument(stage_center, i, &visible_attendees),
            i,
        ));
    }
    // 人気度が高い楽器順のIDリスト生成
    popularity.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    let instruments_ids = popularity.iter().map(|(_, i)| *i).collect::<Vec<_>>();

    // musicianを置く候補となる地点を生成
    let point_generator = PlacementGenerator::new(&input, args.rand_seed);
    let candidates = point_generator.honeycomb_candidates;
    let mut candidates_graph = vec![vec![]; candidates.len()];
    for i in 0..candidates.len() {
        for j in i + 1..candidates.len() {
            if candidates[i].euclidean_distance(&candidates[j]) < 10.0 + 1e-4 {
                candidates_graph[i].push(j);
                candidates_graph[j].push(i);
            }
        }
    }
    println!("generated candidates");

    let mut best_score = -100000000.0;
    let mut rnd = Pcg64Mcg::new(args.rand_seed);
    let mut best_solution = solution.clone();
    println!("initial score: {}", best_score);
    for i in 0..args.iteration {
        let mut available_points: HashSet<usize> = HashSet::from_iter(0..candidates.len());
        let mut used = HashSet::new();
        let mut current_solution = vec![];
        let mut current_solution_mid = vec![];
        // 楽器順に配置
        'outer: for &instrument_id in instruments_ids.iter() {
            println!("check for {instrument_id}");
            let mut count = 0;
            'inner: while count < instruments[&instrument_id] {
                let mut neighbors = HashSet::new();

                // ステージ上の候補地点からランダムに良さそうな箇所を選ぶ
                let mut best_point = available_points.iter().choose(&mut rnd).unwrap().clone();
                let tmp_visible_attendees =
                    input.get_visible_attendees(candidates[best_point], &current_solution);
                let mut best_score = input.raw_score_for_instrument(
                    candidates[best_point],
                    instrument_id,
                    &tmp_visible_attendees,
                );
                for _ in 0..PICK_POINTS_COUNT * 10 {
                    let point = available_points.iter().choose(&mut rnd).unwrap().clone();
                    let tmp_visible_attendees =
                        input.get_visible_attendees(candidates[point], &current_solution);
                    let score = input.raw_score_for_instrument(
                        candidates[point],
                        instrument_id,
                        &tmp_visible_attendees,
                    );
                    if score > best_score {
                        best_score = score;
                        best_point = point;
                    }
                }
                current_solution.push(candidates[best_point]);
                current_solution_mid.push(musician_map[instrument_id][count]);
                used.insert(best_point);
                available_points.remove(&best_point);

                count += 1;
                for &p in &candidates_graph[best_point] {
                    if used.contains(&p) {
                        continue;
                    }
                    neighbors.insert(p);
                }
                while count < instruments[&instrument_id] {
                    if neighbors.is_empty() {
                        println!("couldn't find neighbor");
                        break 'inner;
                    }
                    let pick_count = std::cmp::min(PICK_POINTS_COUNT, neighbors.len());
                    let (best_point, _) = neighbors
                        .iter()
                        .choose_multiple(&mut rnd, pick_count)
                        .into_par_iter()
                        .map(|&point| {
                            let tmp_visible_attendees =
                                input.get_visible_attendees(candidates[point], &current_solution);
                            let score = input.raw_score_for_instrument(
                                candidates[point],
                                instrument_id,
                                &tmp_visible_attendees,
                            );
                            (point, score)
                        })
                        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
                        .unwrap();
                    current_solution.push(candidates[best_point]);
                    current_solution_mid.push(musician_map[instrument_id][count]);
                    used.insert(best_point);
                    available_points.remove(&best_point);
                    neighbors.remove(&best_point);

                    count += 1;
                    for &p in &candidates_graph[best_point] {
                        if used.contains(&p) {
                            continue;
                        }
                        neighbors.insert(p);
                    }
                }
            }
        }
        if current_solution.len() < input.musicians.len() {
            println!("couldn't find solution");
            continue;
        }
        solution.placements = vec![Point::new(0.0, 0.0); input.musicians.len()];
        for i in 0..current_solution.len() {
            solution.placements[current_solution_mid[i]] = current_solution[i];
        }

        match solution.score(&input, full_div) {
            Ok(score) => {
                if score > best_score {
                    best_score = score;
                    best_solution = solution.clone();
                    println!("iter {}, score: {}", i, best_score);
                }
            }
            Err(e) => {
                println!("iter {} error exit: {:?}", i, e);
            }
        }
    }

    std::fs::write(args.output, serde_json::to_string(&best_solution).unwrap()).unwrap();
}
