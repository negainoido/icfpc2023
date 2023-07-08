use anyhow::bail;
use anyhow::Result;
use geo::EuclideanDistance;
use geo::Line;
use geo::Point;
use ordered_float::OrderedFloat;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::f64::consts::PI;

#[derive(Debug, Clone, Copy)]
pub struct Segment {
    p1: Point,
    p2: Point,
}

impl Segment {
    fn dist(self, p: &Point) -> f64 {
        let l = Line::new(self.p1, self.p2);
        let d: Point = l.delta().into();

        if (*p - l.end_point()).dot(d) >= 0.0 {
            return l.end_point().euclidean_distance(p);
        }
        if (*p - l.start_point()).dot(d) <= 0.0 {
            return l.start_point().euclidean_distance(p);
        }

        l.euclidean_distance(p)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Attendee {
    pub x: f64,
    pub y: f64,
    pub tastes: Vec<f64>,
}

type MusicianId = usize;
type AttendeeId = usize;

impl Attendee {
    pub fn pos(&self) -> Point {
        Point::new(self.x, self.y)
    }
}

const BLOCKED_DIST: f64 = 5.0;

#[derive(Debug, Copy, Clone)]
struct AngleInfo {
    dist_sq: OrderedFloat<f64>,
    angle: OrderedFloat<f64>,
    musician_id: usize,
}

impl AngleInfo {
    fn get_covered_angle_range(&self) -> (OrderedFloat<f64>, OrderedFloat<f64>) {
        let asin = (BLOCKED_DIST / self.dist_sq.sqrt()).asin();
        (self.angle - asin, self.angle + asin)
    }
}

pub struct AttendeeScoreDetail {
    pub attendee_id: usize,
    pub matched_musician_ids: Vec<usize>,
    pub score: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Input {
    pub room_width: f64,
    pub room_height: f64,
    pub stage_width: f64,
    pub stage_height: f64,
    pub stage_bottom_left: Point,
    pub musicians: Vec<usize>,
    pub attendees: Vec<Attendee>,
}

impl Input {
    fn in_stage(&self, p: &Point) -> bool {
        const MUSICIAN_CLOSE_DIST: f64 = 10.0;
        p.x() >= self.stage_bottom_left.x() + MUSICIAN_CLOSE_DIST
            && p.x() <= self.stage_bottom_left.x() + self.stage_width - MUSICIAN_CLOSE_DIST
            && p.y() >= self.stage_bottom_left.y() + MUSICIAN_CLOSE_DIST
            && p.y() <= self.stage_bottom_left.y() + self.stage_height - MUSICIAN_CLOSE_DIST
    }

    pub fn is_valid_placements(&self, placements: &Vec<Point>) -> Result<()> {
        if placements.len() != self.musicians.len() {
            bail!(
                "placements.len() != musicians.len(): {} != {}",
                placements.len(),
                self.musicians.len(),
            );
        }

        // Check musicians are in stage
        for i in 0..self.musicians.len() {
            if !self.in_stage(&placements[i]) {
                bail!("{i}-th musician is not in stage: {:?}", placements[i]);
            }
        }

        // Check distance from room walls
        const MUSICIAN_CLOSE_DIST: f64 = 10.0;
        for i in 0..self.musicians.len() {
            let pos = placements[i];
            if !((MUSICIAN_CLOSE_DIST <= pos.x()
                && pos.x() <= self.room_width - MUSICIAN_CLOSE_DIST)
                && (MUSICIAN_CLOSE_DIST <= pos.y()
                    && pos.y() <= self.room_height - MUSICIAN_CLOSE_DIST))
            {
                bail!(
                    "musician {} is too close to room walls: {:?}",
                    i,
                    placements[i],
                );
            }
        }

        // Check ditances between musicians
        for i in 0..(self.musicians.len() - 1) {
            for j in (i + 1)..self.musicians.len() {
                let dist = placements[i].euclidean_distance(&placements[j]);
                if dist < MUSICIAN_CLOSE_DIST {
                    bail!(
                        "musicians {} and {} are too close: {:?} {:?}: dist={dist}",
                        i,
                        j,
                        placements[i],
                        placements[j],
                    );
                }
            }
        }

        Ok(())
    }

    // Impact without considering blocking
    pub fn raw_impact(
        &self,
        attendee_id: AttendeeId,
        musician_id: MusicianId,
        musician_pos: &Point,
    ) -> f64 {
        let instrument = self.musicians[musician_id];
        let attendee = &self.attendees[attendee_id];
        let d = attendee.pos().euclidean_distance(musician_pos);
        ((1_000_000 as f64) * attendee.tastes[instrument] / (d * d)).ceil()
    }

    pub fn impact(
        &self,
        attendee_id: AttendeeId,
        musician_id: MusicianId,
        placements: &Vec<Point>,
    ) -> Result<f64> {
        if placements.len() != self.musicians.len() {
            bail!(
                "placements.len() != musicians.len(): {} != {}",
                placements.len(),
                self.musicians.len(),
            );
        }

        let a_pos = self.attendees[attendee_id].pos();
        let segment = Segment {
            p1: a_pos,
            p2: placements[musician_id],
        };
        for i in 0..placements.len() {
            if i == musician_id {
                continue;
            }

            if segment.dist(&placements[i]) <= BLOCKED_DIST {
                return Ok(0.0);
            }
        }

        Ok(self.raw_impact(attendee_id, musician_id, &placements[musician_id]))
    }

    pub fn score(&self, placements: &Vec<Point>) -> Result<f64> {
        let ans = (0..self.attendees.len())
            .into_par_iter()
            .map(|attendee_id| {
                let mut sum_impact = 0.0;
                for musician_id in 0..self.musicians.len() {
                    sum_impact += self.impact(attendee_id, musician_id, placements).unwrap();
                }
                sum_impact
            })
            .sum();

        Ok(ans)
    }

    pub fn score_attendee_fast(
        &self,
        attendee_id: usize,
        placements: &Vec<Point>,
    ) -> AttendeeScoreDetail {
        let mut sum_impact = 0.0;
        // Compute nearest musician
        let mut nearest_musician_id = 0;
        let mut nearest_distance = OrderedFloat(
            self.attendees[attendee_id]
                .pos()
                .euclidean_distance(&placements[0]),
        );
        for i in 1..placements.len() {
            let distance = OrderedFloat(
                self.attendees[attendee_id]
                    .pos()
                    .euclidean_distance(&placements[i]),
            );
            if distance < nearest_distance {
                nearest_musician_id = i;
                nearest_distance = distance
            }
        }

        // Compute relative angles for each musician based on
        let mut angles = vec![];
        for musician_id in 0..self.musicians.len() {
            let dx: f64 = placements[musician_id].x() - self.attendees[attendee_id].x;
            let dy: f64 = placements[musician_id].y() - self.attendees[attendee_id].y;
            let angle = dy.atan2(dx);
            let dist_sq = dx * dx + dy * dy;
            // angles.push((OrderedFloat(angle), dist_sq, musician_id));
            angles.push(AngleInfo {
                angle: OrderedFloat(angle),
                dist_sq: OrderedFloat(dist_sq),
                musician_id,
            });
        }

        let nearest_musician_angle = angles[nearest_musician_id].angle;
        for musician_id in 0..self.musicians.len() {
            if angles[musician_id].angle >= nearest_musician_angle {
                angles[musician_id].angle -= nearest_musician_angle;
            } else {
                angles[musician_id].angle += 2.0 * PI;
                angles[musician_id].angle -= nearest_musician_angle;
            }
        }
        angles.sort_by_key(|&angle_info| (angle_info.angle, angle_info.dist_sq));
        assert!(angles[0].angle.abs() < 1e-10);

        let mut last_element = angles[0].clone();
        last_element.angle += 2.0 * PI;
        angles.push(last_element);

        let mut is_blocked = vec![false; angles.len()];

        // a(i): musician i's angle from the attendee
        // d(i): musician i's distance from the attendee
        // s(i), e(i):  musician i blocks the range of angle [s(i), e(i)]

        // musician i is blocked iff there exists musician j satisfying at least one of
        // the following two conditions
        // Condition (1)
        //  (1.1) d(i) >= d(j)
        //  (1.2) a(i) >= a(j)
        //  (1.3) a(i) <= e(j)
        // Condition (2)
        //  (1.1) d(i) >= d(j)
        //  (1.2) a(i) >= a(j)
        //  (1.3) a(i) >= s(j)

        let mut max_end_angle_stack = VecDeque::new();
        max_end_angle_stack.push_back((angles[0].dist_sq, angles[0].get_covered_angle_range().1));
        for i in 1..angles.len() {
            let angle_info = angles[i];
            while let Some(&last) = max_end_angle_stack.back() {
                if last.0 > angle_info.dist_sq {
                    max_end_angle_stack.pop_back();
                } else {
                    break;
                }
            }

            let max_end_angle = max_end_angle_stack.back().unwrap().1;
            if max_end_angle >= angle_info.angle {
                is_blocked[i] = true;
            }

            let (_, new_end_angle) = angle_info.get_covered_angle_range();
            if new_end_angle > max_end_angle {
                max_end_angle_stack.push_back((angle_info.dist_sq, new_end_angle));
            }
        }

        // check backward condition
        let mut min_start_angle_stack = VecDeque::new();
        min_start_angle_stack.push_back((
            angles[angles.len() - 1].dist_sq,
            angles[angles.len() - 1].get_covered_angle_range().0,
        ));

        for i in (0..angles.len() - 1).rev() {
            let angle_info = angles[i];
            while let Some(&last) = min_start_angle_stack.back() {
                if last.0 > angle_info.dist_sq {
                    // dbg!(angle_info.dist_sq);
                    // dbg!(min_start_angle_stack.len());
                    min_start_angle_stack.pop_back();
                } else {
                    break;
                }
            }

            let min_start_angle = min_start_angle_stack.back().unwrap().1;
            if min_start_angle <= angle_info.angle {
                is_blocked[i] = true;
            }

            let (new_start_angle, _) = angle_info.get_covered_angle_range();
            if new_start_angle < min_start_angle {
                min_start_angle_stack.push_back((angle_info.dist_sq, new_start_angle));
            }
        }

        let mut matched_musician_ids = vec![];
        for i in 0..angles.len() - 1 {
            if !is_blocked[i] {
                let musician_id = angles[i].musician_id;
                matched_musician_ids.push(musician_id);
                sum_impact += self.raw_impact(attendee_id, musician_id, &placements[musician_id])
            }
        }
        AttendeeScoreDetail {
            score: sum_impact,
            attendee_id,
            matched_musician_ids,
        }
    }

    pub fn score_fast(&self, placements: &Vec<Point>) -> Result<f64> {
        let ans = (0..self.attendees.len())
            // .into_par_iter()
            .into_par_iter()
            .map(|attendee_id| self.score_attendee_fast(attendee_id, placements).score)
            .sum();
        Ok(ans)
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Solution {
    pub placements: Vec<Point>,
}

impl Solution {
    pub fn score(&self, input: &Input) -> Result<f64> {
        // input.score(&self.placements)
        input.is_valid_placements(&self.placements)?;
        input.score_fast(&self.placements)
    }
}

#[cfg(test)]
mod tests {
    use serde_json;

    use super::*;

    #[test]
    fn parse_input() {
        let input = r#"
        {
  "room_width": 2000,
  "room_height": 5000,
  "stage_width": 1000,
  "stage_height": 200,
  "stage_bottom_left": [
    500,
    0
  ],
  "musicians": [
    0,
    1,
    0
  ],
  "attendees": [
    {
      "x": 100,
      "y": 500,
      "tastes": [
        1000,
        -1000
      ]
    },
    {
      "x": 200,
      "y": 1000,
      "tastes": [
        200,
        200
      ]
    },
    {
      "x": 1100,
      "y": 800,
      "tastes": [
        800,
        1500
      ]
    }
  ]
}
        "#;
        let _input: Input = serde_json::from_str(input).unwrap();
    }

    #[test]
    fn parse_solution() {
        let solution = r#"{
  "placements": [
    {
      "x": 590,
      "y": 10
    },
    {
      "x": 1100,
      "y": 100
    },
    {
      "x": 1100,
      "y": 150
    }
  ]
}"#;
        let _solution: Solution = serde_json::from_str(solution).unwrap();
    }

    #[test]
    fn test_segment_dist() {
        let seg = Segment {
            p1: Point::new(0.0, 0.0),
            p2: Point::new(10.0, 0.0),
        };

        assert_eq!(seg.dist(&Point::new(5.0, 5.0)), 5.0);
        assert_eq!(seg.dist(&Point::new(-1.0, 0.0)), 1.0);
        assert!((seg.dist(&Point::new(-1.0, 1.0)) - 2.0f64.sqrt()).abs() < 0.00000001);
    }

    #[test]
    fn sample_eval() {
        let input_str = std::fs::read_to_string("./testdata/sample-input.json").unwrap();
        let input: Input = serde_json::from_str(&input_str).unwrap();
        let solution_str = std::fs::read_to_string("./testdata/sample-output.json").unwrap();
        let solution: Solution = serde_json::from_str(&solution_str).unwrap();
        let score = solution.score(&input).unwrap();
        assert_eq!(score, 5343.0);
    }
}
