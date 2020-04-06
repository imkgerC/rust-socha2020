use crate::action::Action;
use crate::actionlist::{ActionList, ActionListStack};
use crate::gamerules::calculate_legal_moves;
use crate::gamestate::Color::{BLUE, RED};

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum Color {
    RED = 0,
    BLUE = 1,
}
impl Color {
    #[inline(always)]
    pub fn swap(self) -> Color {
        match self {
            RED => BLUE,
            BLUE => RED,
        }
    }
}

#[derive(Clone)]
pub struct GameState {
    pub ply: u8,
    pub color_to_move: Color,

    //Bitboards
    pub occupied: [u128; 2],
    pub pieces: [[u128; 2]; 5],
    pub beetle_stack: [[u128; 2]; 4],
}
impl GameState {
    pub fn new() -> GameState {
        GameState {
            ply: 0,
            color_to_move: RED,
            occupied: [0u128; 2],
            pieces: [[0u128; 2]; 5],
            beetle_stack: [[0u128; 2]; 4],
        }
    }

    pub fn make_action(&self, action: &Action) -> GameState {
        GameState {
            ply: self.ply + 1,
            color_to_move: self.color_to_move.swap(),
            occupied: self.occupied.clone(),
            pieces: self.pieces.clone(),
            beetle_stack: self.beetle_stack.clone(),
        }
    }

    pub fn perft_div(&self, depth: usize) -> u64 {
        self.iperft_root(depth, true)
    }

    pub fn perft(&self, depth: usize) -> u64 {
        self.iperft_root(depth, false)
    }

    fn iperft_root(&self, depth: usize, print: bool) -> u64 {
        if depth == 0 {
            return 1;
        } else {
            let mut als = ActionListStack::with_size(depth + 1);
            calculate_legal_moves(self, als[depth]);
            let mut nc = 0u64;
            for action in als[depth].iter() {
                let next_state = self.make_action(action);
                let n = next_state.iperft(depth - 1, &mut als);
                if print {
                    println!("{:?}: {}", action, n);
                }
                nc += n;
            }
            nc
        }
    }

    fn iperft(&self, depth: usize, als: &mut ActionListStack) -> u64 {
        calculate_legal_moves(self, als[depth]);
        let mut nc = 0u64;
        for action in als[depth].iter() {
            let next_state = self.make_action(action);
            nc += next_state.iperft(depth - 1, als);
        }
        nc
    }
}
