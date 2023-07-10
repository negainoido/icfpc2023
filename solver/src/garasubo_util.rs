use crate::problem::{Input, Solution};
use geo::{EuclideanDistance, Point};
use rand::prelude::{IteratorRandom, SliceRandom};
use rand::Rng;
use rand_pcg::Pcg64Mcg;
use std::collections::{HashSet, VecDeque};

pub fn make_honeycomb_line(
    input: &Input,
    solution: &Solution,
    rnd: &mut Pcg64Mcg,
    musician_map: &Vec<Vec<usize>>,
) -> Solution {
    let target = (0..input.musicians.len()).choose(rnd).unwrap();
    let inst = input.musicians[target];
    let candidates = &musician_map[inst];
    let mut graph = vec![Vec::new(); input.musicians.len()];
    for i in 0..candidates.len() {
        for j in i + 1..candidates.len() {
            let left = candidates[i];
            let right = candidates[j];
            if solution.placements[left].euclidean_distance(&solution.placements[right]) < 15.0 {
                graph[left].push(right);
                graph[right].push(left);
            }
        }
    }
    if graph[target].is_empty() {
        return solution.clone();
    }
    let mut cluster = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(target);
    while let Some(cur) = queue.pop_front() {
        if cluster.contains(&cur) {
            continue;
        }
        cluster.insert(cur);
        if cluster.len() >= 7 {
            break;
        }
        for &next in &graph[cur] {
            queue.push_back(next);
        }
    }
    let cluster = cluster.into_iter().collect::<Vec<_>>();
    if cluster.len() < 2 {
        return solution.clone();
    }

    let min_x = cluster
        .iter()
        .map(|&i| solution.placements[i].x())
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let max_x = cluster
        .iter()
        .map(|&i| solution.placements[i].x())
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let min_y = cluster
        .iter()
        .map(|&i| solution.placements[i].y())
        .min_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();
    let max_y = cluster
        .iter()
        .map(|&i| solution.placements[i].y())
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap();

    let starts = [
        Point::new(min_x, min_y),
        Point::new(min_x, max_y),
        Point::new(max_x, min_y),
        Point::new(max_x, max_y),
    ];
    let mut dir = [[1.0, 0.0], [-1.0, 0.0], [0.0, 1.0], [0.0, -1.0]];
    let dist = 10.0 + rnd.gen_range(0.0..0.5);
    let delta_x = dist / 2.0;
    let delta_y = dist * f64::sqrt(3.0 / 2.0) + 1e-06;
    let deltas = vec![[delta_x, delta_y], [-delta_x, delta_y]];
    dir.shuffle(rnd);
    for &start in starts.iter() {
        for &[dx, dy] in dir.iter() {
            let mut points = vec![];
            let mut x = start.x();
            let mut y = start.y();
            points.push(Point::new(x, y));
            for i in 0..cluster.len() - 1 {
                x += deltas[i % 2][0] * dx;
                y += deltas[i % 2][1] * dx;
                x += deltas[i % 2][1] * dy;
                y += deltas[i % 2][0] * dy;
                points.push(Point::new(x, y));
            }

            let mut new_solution = solution.clone();
            for (&i, &p) in cluster.iter().zip(points.iter()) {
                new_solution.placements[i] = p;
            }
            if input.is_valid_placements(&new_solution.placements).is_ok() {
                println!("try honeycomb line: {:?}", points);
                return new_solution;
            }
        }
    }
    //println!("failed to find honeycomb line");

    solution.clone()
}

pub fn random_swap(
    solution: &Solution,
    musician_map: &Vec<Vec<usize>>,
    rnd: &mut Pcg64Mcg,
) -> (Solution, usize, usize) {
    let mut new_solution = solution.clone();
    let target_insts = (0..musician_map.len()).choose_multiple(rnd, 2);
    let target_insts = (target_insts[0], target_insts[1]);
    let left = *musician_map[target_insts.0].choose(rnd).unwrap();
    let right = *musician_map[target_insts.1].choose(rnd).unwrap();
    new_solution.placements.swap(left, right);

    (new_solution, left, right)
}

pub fn random_move2(input: &Input, solution: &Solution, rnd: &mut Pcg64Mcg) -> (Solution, usize) {
    let target = (0..solution.placements.len()).choose(rnd).unwrap();
    let delta = rnd.gen_range(0.0..1.0);
    let theta = rnd.gen_range(0.0..2.0 * std::f64::consts::PI);
    let mut deltas = vec![
        [delta * f64::cos(theta), delta * f64::sin(theta)],
        [-delta * f64::cos(theta), delta * f64::sin(theta)],
        [delta * f64::cos(theta), -delta * f64::sin(theta)],
        [-delta * f64::cos(theta), -delta * f64::sin(theta)],
    ];

    deltas.shuffle(rnd);
    for &d in deltas.iter() {
        let mut tmp_solution = solution.clone();
        tmp_solution.placements[target] = Point::new(
            solution.placements[target].x() + d[0],
            solution.placements[target].y() + d[1],
        );
        if input.is_valid_placements(&tmp_solution.placements).is_ok() {
            return (tmp_solution, target);
        }
    }
    println!("no valid delta move");

    (solution.clone(), target)
}

// volumeが低いmusicianを遠くに移動させてみる
pub fn random_move3(input: &Input, solution: &Solution, rnd: &mut Pcg64Mcg) -> (Solution, usize) {
    if solution.volumes.is_none() {
        return random_move2(input, solution, rnd);
    }
    let v0_candidates = solution
        .volumes
        .as_ref()
        .unwrap()
        .iter()
        .enumerate()
        .filter(|(_, &v)| v < 1.0)
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    if v0_candidates.is_empty() {
        return random_move2(input, solution, rnd);
    }

    let target = *v0_candidates.choose(rnd).unwrap();
    let delta = rnd.gen_range(10.0..100.0);
    let theta = rnd.gen_range(0.0..2.0 * std::f64::consts::PI);
    let mut deltas = vec![
        [delta * f64::cos(theta), delta * f64::sin(theta)],
        [-delta * f64::cos(theta), delta * f64::sin(theta)],
        [delta * f64::cos(theta), -delta * f64::sin(theta)],
        [-delta * f64::cos(theta), -delta * f64::sin(theta)],
    ];
    deltas.shuffle(rnd);
    for &d in deltas.iter() {
        let mut tmp_solution = solution.clone();
        tmp_solution.placements[target] = Point::new(
            solution.placements[target].x() + d[0],
            solution.placements[target].y() + d[1],
        );
        if input.is_valid_placements(&tmp_solution.placements).is_ok() {
            println!("big delta move");
            return (tmp_solution, target);
        }
    }
    //println!("no valid delta move");

    return (solution.clone(), target);
}

pub fn random_move(
    input: &Input,
    solution: &Solution,
    musician_map: &[Vec<usize>],
    rnd: &mut Pcg64Mcg,
) -> (Solution, usize) {
    let target = (0..solution.placements.len()).choose(rnd).unwrap();
    let inst = input.musicians[target];
    if musician_map[inst].len() == 1 {
        return (solution.clone(), target);
    }
    let mut candidates: HashSet<usize> = HashSet::from_iter(musician_map[inst].iter().cloned());
    candidates.remove(&target);
    let tar2 = candidates.iter().choose(rnd).unwrap();
    let delta = rnd.gen_range(0.0..0.5);
    let mut neighbors = find_neighbor(solution, *tar2, delta);
    neighbors.shuffle(rnd);
    for &n in neighbors.iter() {
        let mut tmp_solution = solution.clone();
        tmp_solution.placements[target] = n;
        if input.is_valid_placements(&tmp_solution.placements).is_ok() {
            return (tmp_solution, target);
        }
    }
    println!("no valid neighbor");

    (solution.clone(), target)
}

fn find_neighbor(solution: &Solution, target: usize, delta: f64) -> Vec<Point> {
    let point = solution.placements[target];
    let mut result = vec![];
    let dist = 10.0 + delta;
    result.push(Point::new(point.x() - dist, point.y()));
    result.push(Point::new(point.x() + dist, point.y()));
    result.push(Point::new(point.x(), point.y() - dist));
    result.push(Point::new(point.x(), point.y() + dist));
    let dx = dist / 2.0;
    let dy = dist * f64::sqrt(3.0 / 2.0) + 1e-06;
    result.push(Point::new(point.x() + dx, point.y() + dy));
    result.push(Point::new(point.x() - dx, point.y() + dy));
    result.push(Point::new(point.x() + dx, point.y() - dy));
    result.push(Point::new(point.x() - dx, point.y() - dy));
    result.push(Point::new(point.x() + dy, point.y() + dx));
    result.push(Point::new(point.x() - dy, point.y() + dx));
    result.push(Point::new(point.x() + dy, point.y() - dx));
    result.push(Point::new(point.x() - dy, point.y() - dx));
    result
}

pub fn switch_volume(solution: &Solution, target: usize) -> Solution {
    let mut new_solution = solution.clone();
    let mut volumes = solution
        .volumes
        .clone()
        .unwrap_or(vec![1.0; solution.placements.len()]);
    if volumes[target] < 1.1 {
        volumes[target] = 10.0;
    } else {
        volumes[target] = 0.0;
    }

    new_solution.volumes = Some(volumes);
    new_solution
}
