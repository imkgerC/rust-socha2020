use crate::action::Action;
use crate::actionlist::ActionListStack;
use crate::bitboard::constants::VALID_FIELDS;
use crate::fieldtype::FieldType;
use crate::gamerules::{calculate_legal_moves, is_game_finished};
use crate::gamestate::Color::{BLUE, RED};
use crate::piece_type::{PieceType, VARIANTS};
use colored::Colorize;
use std::fmt::{Display, Formatter, Result};

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
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
    pub obstacles: u128,
}
impl GameState {
    pub fn new() -> GameState {
        GameState {
            ply: 0,
            color_to_move: RED,
            occupied: [0u128; 2],
            pieces: [[0u128; 2]; 5],
            beetle_stack: [[0u128; 2]; 4],
            obstacles: 0,
        }
    }

    pub fn field_type(&self, index: usize) -> FieldType {
        let field_bb = 1u128 << index;
        debug_assert!(field_bb & VALID_FIELDS != 0);
        if field_bb & self.obstacles != 0u128 {
            FieldType::BLOCKED
        } else {
            for piece_type in VARIANTS.iter() {
                if field_bb
                    & (self.pieces[*piece_type as usize][RED as usize]
                        | self.pieces[*piece_type as usize][BLUE as usize])
                    != 0
                {
                    return FieldType::USED(*piece_type);
                }
            }
            FieldType::FREE
        }
    }
    pub fn is_on_stack(&self, index: usize) -> bool {
        self.is_on_colored_stack(index, Color::RED) || self.is_on_colored_stack(index, Color::BLUE)
    }
    pub fn is_on_colored_stack(&self, index: usize, color: Color) -> bool {
        (1u128 << index) & self.beetle_stack[0][color as usize] != 0
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
                    if (beetle_stack[0][RED as usize] | beetle_stack[0][BLUE as usize]) & from_bit
                        > 0
                    {
                        let mut index = 3;
                        while index > 0 {
                            if beetle_stack[index][self.color_to_move as usize] & from_bit > 0 {
                                beetle_stack[index][self.color_to_move as usize] ^= from_bit;
                                if beetle_stack[index - 1][self.color_to_move as usize] & from_bit
                                    == 0
                                {
                                    // enemy beetle under ours, swap occupancy
                                    occupied[self.color_to_move as usize] ^= from_bit;
                                    occupied[self.color_to_move.swap() as usize] |= from_bit;
                                }
                                break;
                            }
                            index -= 1;
                        }
                        if index == 0 {
                            beetle_stack[0][self.color_to_move as usize] ^= from_bit;
                            let mut own_piece = false;
                            for piece_index in 0..5 {
                                if pieces[piece_index][self.color_to_move as usize] & from_bit
                                    == from_bit
                                {
                                    own_piece = true;
                                    break;
                                }
                            }
                            if !own_piece {
                                // swap occupancy as an enemy piece is now set on this field
                                occupied[self.color_to_move as usize] ^= from_bit;
                                occupied[self.color_to_move.swap() as usize] |= from_bit;
                            }
                        }
                    } else {
                        pieces[PieceType::BEETLE as usize][self.color_to_move as usize] ^= from_bit;
                        occupied[self.color_to_move as usize] ^= from_bit;
                    }
                    let to_bit = 1 << to;
                    if (occupied[RED as usize] | occupied[BLUE as usize]) & to_bit > 0 {
                        // set correct occupancy
                        occupied[self.color_to_move.swap() as usize] &= !to_bit;
                        occupied[self.color_to_move as usize] |= to_bit;
                        for index in 0..4 {
                            if (beetle_stack[index][RED as usize]
                                | beetle_stack[index][BLUE as usize])
                                & to_bit
                                == 0
                            {
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
            obstacles: self.obstacles,
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
        }
        if is_game_finished(self) {
            return 1;
        }
        let mut als = ActionListStack::with_size(depth + 1);
        calculate_legal_moves(self, &mut als[depth]);
        let mut nc = 0u64;
        for i in 0..als[depth].size {
            let action = als[depth][i];
            let next_state = self.make_action(action);
            let n = next_state.iperft(depth - 1, &mut als);
            if print {
                println!("{:?}: {}", action, n);
            }
            nc += n;
        }
        nc
    }

    fn iperft(&self, depth: usize, als: &mut ActionListStack) -> u64 {
        if depth == 0 {
            return 1;
        }
        if is_game_finished(self) {
            return 1;
        }
        calculate_legal_moves(self, &mut als[depth]);
        let mut nc = 0u64;
        for i in 0..als[depth].size {
            let next_state = self.make_action(als[depth][i]);
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
impl Display for GameState {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut res_str = String::new();
        for _ in 0..45 {
            res_str.push_str("-");
        }
        res_str.push_str("\n");
        for y in 0..11isize {
            let y = 10 - y;
            res_str.push_str("|");
            //Fields per row:
            let fields = 11 - (y - 5).abs();
            let extra_spaces = 2 * (11 - fields) - if y != 5 { 1 } else { 0 };
            for _ in 0..extra_spaces {
                res_str.push_str(" ");
            }
            let start_x = (y - 5).max(0);
            if y != 5 {
                res_str.push_str("|")
            }
            for x in start_x..start_x + fields {
                let index = (11 * y + x) as usize;
                //Piecetype
                let field_type = self.field_type(index);
                if let FieldType::USED(pt) = field_type {
                    //Get color of piece type
                    if self.pieces[pt as usize][RED as usize] & (1u128 << index) != 0 {
                        res_str.push_str(&format!(" {} ", field_type.to_string().color("red")));
                    } else {
                        res_str.push_str(&format!(" {} ", field_type.to_string().color("blue")));
                    }
                } else {
                    res_str.push_str(&format!(" {} ", field_type.to_string()));
                }
                if y != 5 || x < 10 {
                    res_str.push_str("|");
                } else {
                }
            }
            for _ in 0..extra_spaces {
                res_str.push_str(" ")
            }
            res_str.push_str("|\n");
        }
        for _ in 0..45 {
            res_str.push_str("-");
        }
        res_str.push_str("\n");
        res_str.push_str(&format!(
            "Ply: {}\nColor to move: {:?}",
            self.ply, self.color_to_move
        ));
        write!(f, "{}", res_str)
    }
}
