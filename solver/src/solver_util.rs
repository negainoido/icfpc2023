use crate::get_time;
use crate::problem::{Input, Segment, Solution};
use geo::{EuclideanDistance, Point};
use rand::Rng;


struct PlayTogetherIndex {

}

impl PlayTogetherIndex {
    fn new(_input: &Input, _placements: &Vec<Point>) -> Self {
        PlayTogetherIndex {

        }
    }

    fn get(&self, _i: usize) -> f64 {
        1.0
    }

    fn move_musician(&mut self, _musician_i: usize, _new_point: Point)  {

    }
}

struct AttendeeIndex {
    musician_point: Point,
    tastes: Vec<f64>,
    angles: Vec<f64>,
    impact: f64,
    cover_counts: Vec<i32>
}

impl AttendeeIndex {
    fn create(musician_id: usize, musician_point: Point, input: &Input) -> Self {
        let mut tastes = vec![];
        let mut angles = vec![];
        let mut impact = 0.0;
        let instrument_id = input.musicians[musician_id];
        for i in 0..input.attendees.len() {
            let taste = input.attendees[i].tastes[instrument_id];
            let distance = input.attendees[i].pos().euclidean_distance(&musician_point);
            let taste = (1000000.0 * taste / (distance * distance)).ceil();
            tastes.push(taste);
            impact += taste;

            let dx = input.attendees[i].x - musician_point.x();
            let dy = input.attendees[i].y - musician_point.y();
            let angle = dy.atan2(dx);
            angles.push(angle);
        }


        AttendeeIndex {
            musician_point,
            tastes,
            angles,
            impact,
            cover_counts: vec![0; input.attendees.len()]
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
        let (angle_left, angle_right) = (angle - asin, angle + asin);

        for i in 0..self.tastes.len() {
            let mut hit = false;
            for offset in -1..=1 {
                let a_angle = self.angles[i] + offset as f64 * std::f64::consts::PI * 2.0;
                if angle_left < a_angle && a_angle < angle_right {
                    hit = true;
                }
            }

            if hit {
                let current = self.cover_counts[i];
                self.cover_counts[i] += value;
                assert!(self.cover_counts[i] >= 0);

                if current == 0 && self.cover_counts[i] > 0 {
                    self.impact -= self.tastes[i];
                }

                if current > 0 && self.cover_counts[i] == 0 {
                    self.impact += self.tastes[i];
                }
            }
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
            let mut attendee_index = AttendeeIndex::create(musician_i, placements[musician_i], input);
            for musician_j in 0..input.musicians.len() {
                if musician_i != musician_j {
                    attendee_index.increase(placements[musician_j]);
                }
            }
            attendee_indexes.push(attendee_index);
        }
        ImpactIndex {input: input.clone(), placements: placements.clone(), attendee_indexes}
    }

    fn get(&self, i: usize) -> f64 {
        self.attendee_indexes[i].get_impact()
    }

    fn move_musician(&mut self, musician_i: usize, new_point: Point)  {
        let old_point = self.placements[musician_i];
        for musician_j in 0..self.input.musicians.len() {
            if musician_i == musician_j {
                continue;
            }
            self.attendee_indexes[musician_j].decrease(old_point);
            self.attendee_indexes[musician_j].increase(new_point);
        }
        self.placements[musician_i] = new_point;
        self.attendee_indexes[musician_i] = AttendeeIndex::create(musician_i, new_point, &self.input);
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
            score += (self.volumes[i] * self.play_together_index.get(i) * self.impact_index.get(i)).ceil();
        }
        score
    }

    fn move_musician(&mut self, musician_i: usize, new_point: Point)  {
        self.play_together_index.move_musician(musician_i, new_point);
        self.impact_index.move_musician(musician_i, new_point);
    }
}


pub fn yamanobori(
    input: &Input,
    best: &mut Vec<Point>,
    best_volume: &Vec<f64>,
    timeout: f64,
    rand_seed: u128,
    reduce_num: usize,
) -> Vec<Point> {
    let mut rng = rand_pcg::Pcg64Mcg::new(rand_seed);
    let input = reduce_attendees(input, reduce_num);
    let mut best_score = input
        .score_fast(&Solution {
            placements: best.clone(),
            volumes: Some(best_volume.clone()),
        })
        .unwrap();

    let mut scoring_index = ScoringIndex::new(&input, best, best_volume);
    dbg!(best_score);
    dbg!(scoring_index.get_score());

    while get_time() < timeout {
        let mut current = best.clone();
        let idx = rng.gen_range(0..best.len());
        let dir = rng.gen_range(0..4);
        let dx = [0.0, 1.0, 0.0, -1.0];
        let dy = [1.0, 0.0, -1.0, 0.0];
        let step = rng.gen_range(1..100) as f64;
        *current[idx].x_mut() += dx[dir] * step;
        *current[idx].y_mut() += dy[dir] * step;

        if input.is_valid_placements(&current).is_err() {
            continue;
        }

        let old_point = best[idx];
        let new_point = current[idx];
        scoring_index.move_musician(idx, new_point);

        // let solution = Solution {
        //     placements: current,
        //     volumes: Some(best_volume.clone()),
        // };
        // let current_score = input.score_fast(&solution);


        // if let Ok(sc) = current_score {
        let sc = scoring_index.get_score();
        if sc > best_score {
            eprintln!(
                "score for reduced attendees is improved (time = {}): {} -> {}",
                get_time(), best_score, sc,
            );
            best_score = sc;
            *best = current;
            // dbg!(scoring_index.get_score());
        } else {
            scoring_index.move_musician(idx, old_point);
        }
        // }
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
    let mut best_score = solution.score(&input).unwrap();

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
            match solution.score(&input) {
                Ok(score) => {
                    if score > best_score {
                        best_score = score;
                        println!("iter {}, score: {}", i, best_score);
                        continue;
                    } else {
                        if let Some(volumes) = &mut solution.volumes {
                            volumes[i] = tmp;
                        }
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
