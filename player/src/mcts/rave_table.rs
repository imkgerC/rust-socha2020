use game_sdk::{Action, Color};

const SET_MOVES: usize = 121 * 6 * 2;
const DRAG_MOVES: usize = 121 * 121 * 2 * 6;

pub struct RaveTable {
    sets: Vec<(f32, f32)>,
    drags: Vec<(f32, f32)>,
    skips: [(f32, f32); 2],
}

impl RaveTable {
    pub fn new() -> Self {
        let sets = vec![(0., 0.); SET_MOVES];
        let drags = vec![(0., 0.); DRAG_MOVES];
        RaveTable {
            sets,
            drags,
            skips: [(0., 0.); 2],
        }
    }

    pub fn get_values(&self, action: Action, color: Color) -> (f32, f32) {
        match action {
            Action::SetMove(piece_type, to) => {
                let mut index = piece_type as usize;
                index *= 121;
                index += to as usize;
                index *= 2;
                index += color as usize;
                *self
                    .sets
                    .get(index)
                    .expect("Did not find set stats at index")
            }
            Action::DragMove(piece_type, from, to) => {
                let mut index = piece_type as usize;
                index *= 121;
                index += to as usize;
                index *= 121;
                index += from as usize;
                index *= 2;
                index += color as usize;
                *self
                    .drags
                    .get(index)
                    .expect("Did not find drag stats at index")
            }
            Action::SkipMove => self.skips[color as usize],
        }
    }

    pub fn add_value(&mut self, action: Action, color: Color, value: f32) {
        match action {
            Action::SetMove(piece_type, to) => {
                let mut index = piece_type as usize;
                index *= 121;
                index += to as usize;
                let mut entry = self
                    .sets
                    .get_mut(index)
                    .expect("Did not find set stats at index");
                entry.0 += value;
                entry.1 += 1.;
            }
            Action::DragMove(piece_type, from, to) => {
                let mut index = piece_type as usize;
                index *= 121;
                index += to as usize;
                index *= 121;
                index += from as usize;
                let mut entry = self
                    .drags
                    .get_mut(index)
                    .expect("Did not find drag stats at index");
                entry.0 += value;
                entry.1 += 1.;
            }
            Action::SkipMove => {
                self.skips[color as usize].0 += value;
                self.skips[color as usize].1 += 1.;
            }
        }
    }
}
