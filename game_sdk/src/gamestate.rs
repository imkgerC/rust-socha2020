use crate::action::Action;
use crate::actionlist::{ActionList, ActionListStack};
use crate::gamerules::calculate_legal_moves;
use crate::gamestate::Color::{BLUE, RED};
use crate::piece_type::PieceType;

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
    pub fn make_action(&self, action: Action) -> GameState {
        let mut pieces = self.pieces.clone();
        let mut occupied = self.occupied.clone();
        let mut beetle_stack = self.beetle_stack.clone();
        match action {
            Action::SkipMove => {}
            Action::DragMove(piece_type, from, to) => {
                if piece_type != PieceType::BEETLE {
                    // unset field
                    debug_assert!(
                        pieces[piece_type as usize][self.color_to_move as usize] & (1 << from) > 0
                    );
                    debug_assert!(occupied[self.color_to_move as usize] & (1 << from) > 0);
                    pieces[piece_type as usize][self.color_to_move as usize] ^= 1 << from;
                    occupied[self.color_to_move as usize] ^= 1 << from;
                    // set field
                    pieces[piece_type as usize][self.color_to_move as usize] |= 1 << to;
                    occupied[self.color_to_move as usize] |= 1 << to;
                } else {
                    let from_bit = 1 << from;
                    if (beetle_stack[0][0] | beetle_stack[0][1]) & from_bit > 0 {
                        for index in 1..4 {
                            if (beetle_stack[index][0] | beetle_stack[index][1]) & from_bit == 0 {
                                debug_assert!(
                                    beetle_stack[index - 1][self.color_to_move as usize] & from_bit
                                        == from_bit
                                );
                                beetle_stack[index - 1][self.color_to_move as usize] ^= from_bit;
                                break;
                            }
                        }
                    } else {
                        pieces[PieceType::BEETLE as usize][self.color_to_move as usize] ^= from_bit;
                        occupied[self.color_to_move as usize] ^= from_bit;
                    }
                    let to_bit = 1 << to;
                    if (occupied[0] | occupied[1]) & to_bit > 0 {
                        for index in 0..4 {
                            if (beetle_stack[index][0] | beetle_stack[index][1]) & to_bit == 0 {
                                beetle_stack[index][self.color_to_move as usize] |= to_bit;
                                break;
                            }
                        }
                    } else {
                        pieces[piece_type as usize][self.color_to_move as usize] |= to_bit;
                        occupied[self.color_to_move as usize] |= to_bit;
                    }
                }
            }
            Action::SetMove(piece_type, to) => {
                pieces[piece_type as usize][self.color_to_move as usize] |= 1 << to;
                occupied[self.color_to_move as usize] |= 1 << to;
            }
        };
        GameState {
            ply: self.ply + 1,
            color_to_move: self.color_to_move.swap(),
            occupied,
            pieces,
            beetle_stack,
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

    pub fn must_player_place_bee(&self) -> bool {
        let round = self.ply / 2;
        if round == 3 {
            if !self.has_player_placed_bee() {
                return true;
            }
        }
        return false;
    }

    pub fn has_player_placed_bee(&self) -> bool {
        return self.pieces[PieceType::BEE as usize][self.color_to_move as usize] > 0;
    }
}
