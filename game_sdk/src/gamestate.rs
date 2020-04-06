use crate::action::Action;
use crate::gamestate::Color::{BLUE, RED};

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Color{
    RED=0,
    BLUE=1
}
impl Color{
    pub fn swap(self) -> Color{
        match self{
            RED=>BLUE,BLUE=>RED
        }
    }
}

#[derive(Clone)]
pub struct GameState{
    pub ply: u8,
    pub color_to_move: Color,

    //Bitboards
    pub occupied: [u128;2],
    pub pieces: [[u128;2];5],
    pub beetle_stack: [[u128;2];4],

}
impl GameState{
    pub fn new() -> GameState{
        GameState{
            ply: 0,
            color_to_move : RED,
            occupied: [0u128;2],
            pieces: [[0u128;2];5],
            beetle_stack: [[0u128;2];4]
        }
    }
    pub fn make_action(&self, action: &Action) -> GameState{
        GameState{
            ply: self.ply +1,
            color_to_move: self.color_to_move.swap(),
            occupied: self.occupied.clone(),
            pieces: self.pieces.clone(),
            beetle_stack: self.beetle_stack.clone()
        }
    }

    pub fn perft_div(&self, depth: usize) -> u64{

    }

    pub fn perft(&self, depth: usize) -> u64{

    }
}