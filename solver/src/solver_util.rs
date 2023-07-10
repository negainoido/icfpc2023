use crate::get_time;
use crate::problem::{Input, Segment, Solution};
use geo::{EuclideanDistance, Point};
use ordered_float::OrderedFloat;
use rand::Rng;

struct PlayTogetherIndex {}

impl PlayTogetherIndex {
    fn new(_input: &Input, _placements: &Vec<Point>) -> Self {
        PlayTogetherIndex {}
    }

    fn get(&self, _i: usize) -> f64 {
        1.0
    }

    fn move_musician(&mut self, _musician_i: usize, _new_point: Point) {}
}

struct AttendeeIndex {
    musician_point: Point,
    tastes: Vec<f64>,
    sorted_angles: Vec<(OrderedFloat<f64>, usize)>,
    impact: f64,
    cover_counts: Vec<i32>,
}

impl AttendeeIndex {
    fn create(musician_id: usize, musician_point: Point, input: &Input) -> Self {
        let mut tastes = vec![];
        // let mut angles = vec![];
        let mut impact = 0.0;
        let instrument_id = input.musicians[musician_id];

        let mut sorted_angles = vec![];
        for i in 0..input.attendees.len() {
            let taste = input.attendees[i].tastes[instrument_id];
            let distance = input.attendees[i].pos().euclidean_distance(&musician_point);
            let taste = (1000000.0 * taste / (distance * distance)).ceil();
            tastes.push(taste);
            impact += taste;

            let dx = input.attendees[i].x - musician_point.x();
            let dy = input.attendees[i].y - musician_point.y();
            let angle = dy.atan2(dx);
            // angles.push(angle);

            sorted_angles.push((OrderedFloat(angle) - 2.0 * std::f64::consts::PI, i));
            sorted_angles.push((OrderedFloat(angle), i));
            sorted_angles.push((OrderedFloat(angle) + 2.0 * std::f64::consts::PI, i));
        }
        // 番兵
        sorted_angles.push((OrderedFloat(-4.0 * std::f64::consts::PI), 0));
        sorted_angles.push((OrderedFloat(4.0 * std::f64::consts::PI), 0));
        sorted_angles.sort();

        AttendeeIndex {
            musician_point,
            tastes,
            sorted_angles,
            impact,
            cover_counts: vec![0; input.attendees.len()],
        }
    }

    fn decrease(&mut self, point: Point) {
        self.add(point, -1);
    }

    fn increase(&mut self, point: Point) {
        self.add(point, 1)
    }

    fn add(&mut self, point: Point, value: i32) {
        let dx = point.x() - self.musician_point.x();
        let dy = point.y() - self.musician_point.y();
        let distance = point.euclidean_distance(&self.musician_point);
        let angle = dy.atan2(dx);
        let asin = (5.0 / distance).asin();
        let (angle_left, angle_right) = (OrderedFloat(angle - asin), OrderedFloat(angle + asin));

        let mut lb = 0;
        let mut ub = self.sorted_angles.len();
        while ub - lb > 1 {
            let mb = (ub + lb) / 2;
            if self.sorted_angles[mb].0 <= angle_left {
                lb = mb;
            } else {
                ub = mb;
            }
        }
        let mut cursor = ub;
        while cursor < self.sorted_angles.len() && self.sorted_angles[cursor].0 < angle_right {
            let index = self.sorted_angles[cursor].1;
            let current = self.cover_counts[index];
            self.cover_counts[index] += value;
            assert!(self.cover_counts[index] >= 0);

            if current == 0 && self.cover_counts[index] > 0 {
                self.impact -= self.tastes[index];
            }

            if current > 0 && self.cover_counts[index] == 0 {
                self.impact += self.tastes[index];
            }
            cursor += 1;
        }
    }

    fn get_impact(&self) -> f64 {
        self.impact
    }
}

struct ImpactIndex {
    input: Input,
    placements: Vec<Point>,
    attendee_indexes: Vec<AttendeeIndex>,
}

impl ImpactIndex {
    fn new(input: &Input, placements: &Vec<Point>) -> Self {
        let mut attendee_indexes = vec![];
        for musician_i in 0..input.musicians.len() {
            let mut attendee_index =
                AttendeeIndex::create(musician_i, placements[musician_i], input);
            for musician_j in 0..input.musicians.len() {
                if musician_i != musician_j {
                    attendee_index.increase(placements[musician_j]);
                }
            }
            attendee_indexes.push(attendee_index);
        }
        ImpactIndex {
            input: input.clone(),
            placements: placements.clone(),
            attendee_indexes,
        }
    }

    fn get(&self, i: usize) -> f64 {
        self.attendee_indexes[i].get_impact()
    }

    fn move_musician(&mut self, musician_i: usize, new_point: Point) {
        let old_point = self.placements[musician_i];
        for musician_j in 0..self.input.musicians.len() {
            if musician_i == musician_j {
                continue;
            }
            self.attendee_indexes[musician_j].decrease(old_point);
            self.attendee_indexes[musician_j].increase(new_point);
        }
        self.placements[musician_i] = new_point;
        self.attendee_indexes[musician_i] =
            AttendeeIndex::create(musician_i, new_point, &self.input);
        for musician_j in 0..self.input.musicians.len() {
            if musician_i != musician_j {
                self.attendee_indexes[musician_i].increase(self.placements[musician_j]);
            }
        }
    }
}

struct ScoringIndex {
    input: Input,
    volumes: Vec<f64>,
    play_together_index: PlayTogetherIndex,
    impact_index: ImpactIndex,
}

impl ScoringIndex {
    fn new(input: &Input, placements: &Vec<Point>, volumes: &Vec<f64>) -> Self {
        assert_eq!(input.pillars.len(), 0);

        // let mut cover_counts = vec![vec![0; input.attendees.len()]; input.musicians.len()];
        // for musician_i in 0..input.musicians.len() {
        //     let place_i = placements[musician_i];
        //
        //     for attendee_id in 0..input.attendees.len() {
        //         let dx = input.attendees[attendee_id].x - place_i.x();
        //         let dy = input.attendees[attendee_id].y - place_i.y();
        //     }
        // }
        //
        ScoringIndex {
            input: input.clone(),
            // placements: placements.clone(),
            volumes: volumes.clone(),
            // cover_counts,
            play_together_index: PlayTogetherIndex::new(input, placements),
            impact_index: ImpactIndex::new(input, placements),
        }
    }

    fn get_score(&self) -> f64 {
        let mut score = 0.0;
        for i in 0..self.input.musicians.len() {
            score += (self.volumes[i] * self.play_together_index.get(i) * self.impact_index.get(i))
                .ceil();
        }
        score
    }

    fn is_valid_move(&self, musician_i: usize, new_point: Point) -> bool {
        // Check musician is in stage
        if !self.input.in_stage(&new_point) {
            return false;
        }

        // Check distance from room walls
        const MUSICIAN_CLOSE_DIST: f64 = 10.0;
        if !((MUSICIAN_CLOSE_DIST <= new_point.x()
            && new_point.x() <= self.input.room_width - MUSICIAN_CLOSE_DIST)
            && (MUSICIAN_CLOSE_DIST <= new_point.y()
                && new_point.y() <= self.input.room_height - MUSICIAN_CLOSE_DIST))
        {
            return false;
        }

        // Check ditances between musicians
        for musician_j in 0..self.input.musicians.len() {
            if musician_i == musician_j {
                continue;
            }

            let dist = new_point.euclidean_distance(&self.impact_index.placements[musician_j]);
            if dist < MUSICIAN_CLOSE_DIST {
                return false;
            }
        }
        true
    }

    fn move_musician(&mut self, musician_i: usize, new_point: Point) -> bool {
        if self.is_valid_move(musician_i, new_point) {
            self.play_together_index
                .move_musician(musician_i, new_point);
            self.impact_index.move_musician(musician_i, new_point);
            true
        } else {
            false
        }
    }
}

pub fn yamanobori(
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

    let mut scoring_index = ScoringIndex::new(&input, best, &best_volume.to_vec());
    dbg!(best_score);
    dbg!(scoring_index.get_score());
    let mut count = 0;

    while get_time() < timeout {
        count += 1;
        let mut current = best.clone();
        let idx = rng.gen_range(0..best.len());
        let dir = rng.gen_range(0..4);
        let dx = [0.0, 1.0, 0.0, -1.0];
        let dy = [1.0, 0.0, -1.0, 0.0];
        let step = rng.gen_range(1..100) as f64;
        *current[idx].x_mut() += dx[dir] * step;
        *current[idx].y_mut() += dy[dir] * step;

        // if input.is_valid_placements(&current).is_err() {
        //     continue;
        // }

        let old_point = best[idx];
        let new_point = current[idx];
        let move_success = scoring_index.move_musician(idx, new_point);
        if !move_success {
            continue;
        }

        let sc = scoring_index.get_score();
        if sc > best_score {
            eprintln!(
                "score for reduced attendees is improved (time = {}, count = {}): {} -> {}",
                get_time(),
                count,
                best_score,
                sc,
            );
            best_score = sc;
            *best = current;
            // dbg!(scoring_index.get_score());
        } else {
            scoring_index.move_musician(idx, old_point);
        }
    }
    best.to_vec()
}

pub fn reduce_attendees(input: &Input, num: usize) -> Input {
    let mut new_attendees = vec![];
    for attendee in &input.attendees {
        let mut min_dist: f64 = 1.0e9;
        for segment in [
            Segment {
                p1: input.stage_bottom_left,
                p2: input.stage_bottom_left + Point::new(input.stage_width, 0.0),
            },
            Segment {
                p1: input.stage_bottom_left,
                p2: input.stage_bottom_left + Point::new(0.0, input.stage_height),
            },
            Segment {
                p1: input.stage_bottom_left + Point::new(0.0, input.stage_height),
                p2: input.stage_bottom_left + Point::new(input.stage_width, input.stage_height),
            },
            Segment {
                p1: input.stage_bottom_left + Point::new(input.stage_width, 0.0),
                p2: input.stage_bottom_left + Point::new(input.stage_width, input.stage_height),
            },
        ] {
            min_dist = min_dist.min(segment.dist(&attendee.pos()));
        }
        new_attendees.push((min_dist, attendee));
    }

    new_attendees.sort_by_key(|k| ordered_float::OrderedFloat(k.0));
    new_attendees.truncate(num);
    let mut input = input.clone();
    input.attendees = new_attendees.into_iter().map(|v| v.1.clone()).collect();
    input
}

pub fn volume_optimize(input: &Input, solution: &Solution) -> Solution {
    let mut solution = solution.clone();
    let mut best_score = solution.score(input).unwrap();

    let original_volumes = solution
        .volumes
        .clone()
        .unwrap_or(vec![1.0; input.musicians.len()]);
    solution.volumes = Some(original_volumes);

    // Volume optimize
    for i in 0..input.musicians.len() {
        for vol in [0.0, 0.1, 9.9, 10.0] {
            let tmp = solution.volumes.as_ref().map(|v| v[i]).unwrap_or(1.0);
            if let Some(volumes) = &mut solution.volumes {
                volumes[i] = vol;
            }
            match solution.score(input) {
                Ok(score) => {
                    if score > best_score {
                        best_score = score;
                        println!("iter {}, score: {}", i, best_score);
                        continue;
                    } else if let Some(volumes) = &mut solution.volumes {
                        volumes[i] = tmp;
                    }
                }
                Err(e) => {
                    println!("iter {} error exit: {:?}", i, e);
                }
            }
        }
    }
    solution
}
