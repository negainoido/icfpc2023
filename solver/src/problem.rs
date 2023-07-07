use anyhow::bail;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Pos {
    pub x: f64,
    pub y: f64,
}

impl Pos {
    pub fn dist_sq(&self, other: &Pos) -> f64 {
        (self.x - other.x).powi(2) + (self.y - other.y).powi(2)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Attendee {
    pub x: f64,
    pub y: f64,
    pub tastes: Vec<f64>,
}

type MusicianId = usize;
type AttendeeId = usize;

impl Attendee {
    pub fn taste(&self, musician: MusicianId) -> f64 {
        self.tastes[musician]
    }

    pub fn pos(&self) -> Pos {
        Pos {
            x: self.x,
            y: self.y,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Input {
    pub room_width: f64,
    pub room_height: f64,
    pub stage_width: f64,
    pub stage_height: f64,
    pub stage_bottom_left: Pos,
    pub musicians: Vec<usize>,
    pub attendees: Vec<Attendee>,
}

impl Input {
    fn in_stage(&self, p: &Pos) -> bool {
        p.x >= self.stage_bottom_left.x
            && p.x <= self.stage_bottom_left.x + self.stage_width
            && p.y >= self.stage_bottom_left.y
            && p.y <= self.stage_bottom_left.y + self.stage_height
    }

    pub fn is_valid_placements(&self, placements: &Vec<Pos>) -> Result<()> {
        if placements.len() != self.musicians.len() {
            bail!(
                "placements.len() != musicians.len(): {} != {}",
                placements.len(),
                self.musicians.len(),
            );
        }

        for i in 0..self.musicians.len() {
            if !self.in_stage(&placements[i]) {
                bail!("{i}-th musician is not in stage: {:?}", placements[i]);
            }
        }

        const MUSICIAN_SPACE_DIST_SQ: f64 = 100.0;
        for i in 0..(self.musicians.len() - 1) {
            for j in (i + 1)..self.musicians.len() {
                let dist2 = placements[i].dist_sq(&placements[j]);
                if dist2 <= MUSICIAN_SPACE_DIST_SQ {
                    bail!(
                        "musicians {} and {} are too close: {:?} {:?}: dist**2={dist2}",
                        i,
                        j,
                        placements[i],
                        placements[j],
                    );
                }
            }
        }

        // TODO(udon): distance from room walls

        Ok(())
    }

    // Impact without considering blocking
    pub fn raw_impact(
        &self,
        attendee_id: AttendeeId,
        musician_id: MusicianId,
        musician_pos: &Pos,
    ) -> u64 {
        let instrument = self.musicians[musician_id];
        let attendee = &self.attendees[attendee_id];
        let d2 = attendee.pos().dist_sq(musician_pos);
        ((1_000_000 as f64) * attendee.tastes[instrument] / d2).ceil() as u64
    }

    pub fn impact(
        &self,
        attendee_id: AttendeeId,
        musician_id: MusicianId,
        placements: &Vec<Pos>,
    ) -> Result<u64> {
        if placements.len() != self.musicians.len() {
            bail!(
                "placements.len() != musicians.len(): {} != {}",
                placements.len(),
                self.musicians.len(),
            );
        }

        //const BLOCKED_DIST_SQ: f64 = 25.0;
        let is_blocked = false;

        // TODO(udon): blocking check

        if is_blocked {
            Ok(0)
        } else {
            Ok(self.raw_impact(attendee_id, musician_id, &placements[musician_id]))
        }
    }

    pub fn score(&self, placements: &Vec<Pos>) -> Result<u64> {
        let mut sum_impact = 0;
        for attendee_id in 0..self.attendees.len() {
            for musician_id in 0..self.musicians.len() {
                sum_impact += self.impact(attendee_id, musician_id, placements)?;
            }
        }
        Ok(sum_impact)
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Solution {
    pub placements: Vec<Pos>,
}

impl Solution {
    pub fn score(&self, input: &Input) -> Result<u64> {
        input.score(&self.placements)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json;

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
}
