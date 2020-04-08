use crate::action::Action;
use crate::actionlist::ActionListStack;
use crate::bitboard::constants::VALID_FIELDS;
use crate::fieldtype::FieldType;
use crate::gamerules::{calculate_legal_moves, is_game_finished};
use crate::gamestate::Color::{BLUE, RED};
use crate::hashing::{BEETLE_STACK_HASH, COLOR_TO_MOVE_HASH, PIECE_HASH};
use crate::piece_type::{PieceType, PIECETYPE_VARIANTS};
use colored::Colorize;
use std::fmt::{Display, Formatter, Result};

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq)]
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

    pub fn to_string(&self) -> String {
        match self {
            Color::RED => "RED".to_string(),
            Color::BLUE => "BLUE".to_string(),
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
    pub hash: u64,
}
impl GameState {
    pub fn to_fen(&self) -> String {
        let mut fen = String::new();
        fen.push_str(&format!(
            "{} {:?} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
            self.ply,
            self.color_to_move,
            self.obstacles,
            self.beetle_stack[0][0],
            self.beetle_stack[1][0],
            self.beetle_stack[2][0],
            self.beetle_stack[3][0],
            self.beetle_stack[0][1],
            self.beetle_stack[1][1],
            self.beetle_stack[2][1],
            self.beetle_stack[3][1],
            self.pieces[0][0],
            self.pieces[1][0],
            self.pieces[2][0],
            self.pieces[3][0],
            self.pieces[4][0],
            self.pieces[0][1],
            self.pieces[1][1],
            self.pieces[2][1],
            self.pieces[3][1],
            self.pieces[4][1]
        ));
        fen
    }
    pub fn from_fen(fen: String) -> GameState {
        let mut entries: Vec<&str> = fen.split(" ").collect();
        assert_eq!(entries.len(), 21);
        let ply = entries.remove(0).parse::<u8>().unwrap();
        let color_to_move = match entries.remove(0) {
            "red" | "RED" => RED,
            "blue" | "BLUE" => BLUE,
            _ => panic!("Invalid Color"),
        };
        let obstacles = entries.remove(0).parse::<u128>().unwrap();
        let mut beetle_stack = [[0u128; 2]; 4];
        for i in 0..2 {
            for j in 0..4 {
                beetle_stack[j][i] = entries.remove(0).parse::<u128>().unwrap();
            }
        }
        let mut pieces = [[0u128; 2]; 5];
        for i in 0..2 {
            for j in 0..5 {
                pieces[j][i] = entries.remove(0).parse::<u128>().unwrap();
            }
        }
        let hash = GameState::calculate_hash(&pieces, color_to_move, &beetle_stack);
        let mut occupied = [0u128; 2];
        for index in 0..128 {
            if (beetle_stack[0][RED as usize] | beetle_stack[0][BLUE as usize]) & 1u128 << index > 0
            {
                let mut i: isize = 3;
                while i >= 0 {
                    if beetle_stack[i as usize][RED as usize] & 1u128 << index > 0 {
                        occupied[RED as usize] |= 1u128 << index;
                    } else if beetle_stack[i as usize][BLUE as usize] & 1u128 << index > 0 {
                        occupied[BLUE as usize] |= 1u128 << index;
                    }
                    i -= 1;
                }
            } else {
                if (pieces[0][RED as usize]
                    | pieces[1][RED as usize]
                    | pieces[2][RED as usize]
                    | pieces[3][RED as usize]
                    | pieces[4][RED as usize])
                    & 1u128 << index
                    > 0
                {
                    occupied[RED as usize] ^= 1u128 << index
                } else if (pieces[0][BLUE as usize]
                    | pieces[1][BLUE as usize]
                    | pieces[2][BLUE as usize]
                    | pieces[3][BLUE as usize]
                    | pieces[4][BLUE as usize])
                    & 1u128 << index
                    > 0
                {
                    occupied[BLUE as usize] ^= 1u128 << index;
                }
            }
        }
        GameState {
            ply,
            color_to_move,
            pieces,
            occupied,
            hash,
            beetle_stack,
            obstacles,
        }
    }
    pub fn new() -> GameState {
        let pieces = [[0u128; 2]; 5];
        let beetle_stack = [[0u128; 2]; 4];
        let hash = GameState::calculate_hash(&pieces, RED, &beetle_stack);
        GameState {
            ply: 0,
            color_to_move: RED,
            occupied: [0u128; 2],
            pieces,
            beetle_stack,
            obstacles: 0,
            hash,
        }
    }
    pub fn calculate_hash(
        pieces: &[[u128; 2]; 5],
        color_to_move: Color,
        beetle_stack: &[[u128; 2]; 4],
    ) -> u64 {
        let mut hash = 0u64;
        if color_to_move == RED {
            hash ^= COLOR_TO_MOVE_HASH;
        }
        for &piece_type in PIECETYPE_VARIANTS.iter() {
            for &color in [Color::RED, Color::BLUE].iter() {
                let mut piece_bb = pieces[piece_type as usize][color as usize];
                while piece_bb > 0 {
                    let index = piece_bb.trailing_zeros();
                    piece_bb ^= 1u128 << index;
                    hash ^= PIECE_HASH[piece_type as usize][color as usize][index as usize];
                }
            }
        }
        for b_index in 0..4 {
            for &color in [Color::RED, Color::BLUE].iter() {
                let mut beetle_bb = beetle_stack[b_index][color as usize];
                while beetle_bb > 0 {
                    let index = beetle_bb.trailing_zeros();
                    beetle_bb ^= 1u128 << index;
                    hash ^= BEETLE_STACK_HASH[b_index][color as usize][index as usize];
                }
            }
        }
        hash
    }

    pub fn field_type(&self, index: usize) -> FieldType {
        let field_bb = 1u128 << index;
        debug_assert!(field_bb & VALID_FIELDS != 0);
        if field_bb & self.obstacles != 0u128 {
            FieldType::BLOCKED
        } else {
            for piece_type in PIECETYPE_VARIANTS.iter() {
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

    #[inline(always)]
    pub fn pieces_from_color(&self, color: Color) -> u128 {
        self.pieces[PieceType::BEE as usize][color as usize]
            | self.pieces[PieceType::SPIDER as usize][color as usize]
            | self.pieces[PieceType::GRASSHOPPER as usize][color as usize]
            | self.pieces[PieceType::BEETLE as usize][color as usize]
            | self.pieces[PieceType::ANT as usize][color as usize]
    }
    #[inline(always)]
    pub fn occupied(&self) -> u128 {
        self.occupied[Color::RED as usize] | self.occupied[Color::BLUE as usize]
    }

    #[inline(always)]
    pub fn is_on_stack(&self, index: usize) -> bool {
        self.is_on_colored_stack(index, Color::RED) || self.is_on_colored_stack(index, Color::BLUE)
    }

    #[inline(always)]
    pub fn is_on_colored_stack(&self, index: usize, color: Color) -> bool {
        (1u128 << index) & self.beetle_stack[0][color as usize] != 0
    }

    pub fn bb_to_string(bb: u128) -> String {
        let mut state = GameState::new();
        state.obstacles = bb;
        format!("{}", state)
    }
    #[inline(always)]
    pub fn make_action(&mut self, action: Action) {
        match action {
            Action::SkipMove => {}
            Action::DragMove(piece_type, from, to) => {
                if piece_type != PieceType::BEETLE {
                    // unset field
                    debug_assert!(
                        self.pieces[piece_type as usize][self.color_to_move as usize] & (1 << from)
                            > 0
                    );
                    debug_assert!(self.occupied[self.color_to_move as usize] & (1 << from) > 0);
                    self.pieces[piece_type as usize][self.color_to_move as usize] ^= 1 << from;
                    self.occupied[self.color_to_move as usize] ^= 1 << from;
                    self.hash ^=
                        PIECE_HASH[piece_type as usize][self.color_to_move as usize][from as usize];
                    // set field
                    debug_assert!(
                        self.pieces[piece_type as usize][self.color_to_move as usize] & (1 << to)
                            == 0
                    );
                    debug_assert!((self.occupied() | self.obstacles) & (1 << to) == 0);
                    self.pieces[piece_type as usize][self.color_to_move as usize] ^= 1 << to;
                    self.occupied[self.color_to_move as usize] ^= 1 << to;
                    self.hash ^=
                        PIECE_HASH[piece_type as usize][self.color_to_move as usize][to as usize];
                } else {
                    let from_bit = 1 << from;
                    if self.is_on_stack(from as usize) {
                        let mut index = 3;
                        while index > 0 {
                            if self.beetle_stack[index][self.color_to_move as usize] & from_bit > 0
                            {
                                self.beetle_stack[index][self.color_to_move as usize] ^= from_bit;
                                self.hash ^= BEETLE_STACK_HASH[index][self.color_to_move as usize]
                                    [from as usize];
                                if self.beetle_stack[index - 1][self.color_to_move as usize]
                                    & from_bit
                                    == 0
                                {
                                    // enemy beetle under ours, swap occupancy
                                    debug_assert!(
                                        (self.occupied[self.color_to_move as usize] & from_bit)
                                            .count_ones()
                                            == 1
                                    );
                                    debug_assert!(
                                        self.occupied[self.color_to_move.swap() as usize]
                                            & from_bit
                                            == 0
                                    );
                                    self.occupied[self.color_to_move as usize] ^= from_bit;
                                    self.occupied[self.color_to_move.swap() as usize] ^= from_bit;
                                }
                                break;
                            }
                            debug_assert!(
                                self.beetle_stack[index][self.color_to_move.swap() as usize]
                                    & from_bit
                                    == 0
                            ); //Make sure our beetle is actually on top of the stack
                            index -= 1;
                        }
                        if index == 0 {
                            debug_assert!(
                                (self.beetle_stack[0][self.color_to_move as usize] & from_bit)
                                    .count_ones()
                                    == 1
                            );
                            self.beetle_stack[0][self.color_to_move as usize] ^= from_bit;
                            self.hash ^=
                                BEETLE_STACK_HASH[0][self.color_to_move as usize][from as usize];
                            let own_piece =
                                self.pieces_from_color(self.color_to_move) & from_bit == from_bit;
                            if !own_piece {
                                // swap occupancy as an enemy piece is now set on this field
                                debug_assert!(
                                    self.occupied[self.color_to_move as usize] & from_bit
                                        == from_bit
                                );
                                debug_assert!(
                                    self.occupied[self.color_to_move.swap() as usize] & from_bit
                                        == 0
                                );
                                self.occupied[self.color_to_move as usize] ^= from_bit;
                                self.occupied[self.color_to_move.swap() as usize] ^= from_bit;
                            }
                        }
                    } else {
                        debug_assert!(
                            self.pieces[PieceType::BEETLE as usize][self.color_to_move as usize]
                                & from_bit
                                == from_bit
                        );
                        debug_assert!(
                            self.occupied[self.color_to_move as usize] & from_bit == from_bit
                        );
                        self.pieces[PieceType::BEETLE as usize][self.color_to_move as usize] ^=
                            from_bit;
                        self.occupied[self.color_to_move as usize] ^= from_bit;
                        self.hash ^= PIECE_HASH[PieceType::BEETLE as usize]
                            [self.color_to_move as usize][from as usize];
                    }

                    let to_bit = 1 << to;
                    if (self.occupied()) & to_bit > 0 {
                        //Set on stack
                        // set correct occupancy
                        //We don't know what color we are sitting on
                        self.occupied[self.color_to_move.swap() as usize] &= !to_bit;
                        self.occupied[self.color_to_move as usize] |= to_bit;

                        for index in 0..4 {
                            if (self.beetle_stack[index][RED as usize]
                                | self.beetle_stack[index][BLUE as usize])
                                & to_bit
                                == 0
                            {
                                self.beetle_stack[index][self.color_to_move as usize] ^= to_bit;
                                self.hash ^= BEETLE_STACK_HASH[index][self.color_to_move as usize]
                                    [to as usize];
                                break;
                            }
                        }
                    } else {
                        //Normal move
                        debug_assert!(
                            self.pieces[PieceType::BEETLE as usize][self.color_to_move as usize]
                                & to_bit
                                == 0
                        );
                        debug_assert!(self.occupied[self.color_to_move as usize] & to_bit == 0);
                        self.pieces[PieceType::BEETLE as usize][self.color_to_move as usize] ^=
                            to_bit;
                        self.occupied[self.color_to_move as usize] ^= to_bit;
                        self.hash ^= PIECE_HASH[PieceType::BEETLE as usize]
                            [self.color_to_move as usize][to as usize];
                    }
                }
            }
            Action::SetMove(piece_type, to) => {
                debug_assert!((self.occupied() | self.obstacles) & (1 << to) == 0);
                self.pieces[piece_type as usize][self.color_to_move as usize] ^= 1 << to;
                self.occupied[self.color_to_move as usize] ^= 1 << to;
                self.hash ^=
                    PIECE_HASH[piece_type as usize][self.color_to_move as usize][to as usize];
            }
        };
        self.ply += 1;
        self.color_to_move = self.color_to_move.swap();
        self.hash ^= COLOR_TO_MOVE_HASH;
        debug_assert_eq!(
            self.hash,
            GameState::calculate_hash(&self.pieces, self.color_to_move, &self.beetle_stack)
        );
    }

    pub fn unmake_action(&mut self, action: Action) {
        self.color_to_move = self.color_to_move.swap();
        self.hash ^= COLOR_TO_MOVE_HASH;
        match action {
            Action::SkipMove => {}
            Action::DragMove(piece_type, from, to) => {
                self.ply -= 1;
                self.make_action(Action::DragMove(piece_type, to, from));
                self.hash ^= COLOR_TO_MOVE_HASH;
                self.color_to_move = self.color_to_move.swap();
            }
            Action::SetMove(piece_type, to) => {
                debug_assert!(
                    ((self.pieces[piece_type as usize][self.color_to_move as usize]) & (1 << to))
                        .count_ones()
                        == 1
                );
                debug_assert!(
                    ((self.occupied[self.color_to_move as usize]) & (1 << to)).count_ones() == 1
                );
                self.pieces[piece_type as usize][self.color_to_move as usize] ^= 1 << to;
                self.occupied[self.color_to_move as usize] ^= 1 << to;
                self.hash ^=
                    PIECE_HASH[piece_type as usize][self.color_to_move as usize][to as usize];
            }
        };
        self.ply -= 1;
        debug_assert_eq!(
            self.hash,
            GameState::calculate_hash(&self.pieces, self.color_to_move, &self.beetle_stack)
        );
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
        let mut state = self.clone();
        let mut als = ActionListStack::with_size(depth + 1);
        calculate_legal_moves(self, &mut als[depth]);
        let mut nc = 0u64;
        for i in 0..als[depth].size {
            let action = als[depth][i];
            state.make_action(action);
            let n = state.iperft(depth - 1, &mut als);
            if print {
                println!("{:?}: {}", action, n);
            }
            state.unmake_action(action);
            nc += n;
        }
        nc
    }

    fn iperft(&mut self, depth: usize, als: &mut ActionListStack) -> u64 {
        if depth == 0 {
            return 1;
        }
        if is_game_finished(self) {
            return 1;
        }
        calculate_legal_moves(self, &mut als[depth]);
        let mut nc = 0u64;
        for i in 0..als[depth].size {
            self.make_action(als[depth][i]);
            nc += self.iperft(depth - 1, als);
            self.unmake_action(als[depth][i]);
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
        let mut stack_strings: Vec<String> = Vec::new();
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
                    let is_on_stack = self.is_on_stack(index);
                    if is_on_stack {
                        let mut stack_str = String::new();
                        for i in 0..4 {
                            if (self.beetle_stack[i][RED as usize] & (1u128 << index)) > 0 {
                                if i > 0 {
                                    stack_str.push_str(&format!(
                                        "<-{}",
                                        PieceType::BEETLE.to_string().color("red")
                                    ));
                                } else {
                                    stack_str.push_str(&format!(
                                        "{}",
                                        PieceType::BEETLE.to_string().color("red")
                                    ));
                                }
                            } else if (self.beetle_stack[i][BLUE as usize] & (1u128 << index)) > 0 {
                                if i > 0 {
                                    stack_str.push_str(&format!(
                                        "<-{}",
                                        PieceType::BEETLE.to_string().color("blue")
                                    ));
                                } else {
                                    stack_str.push_str(&format!(
                                        "{}",
                                        PieceType::BEETLE.to_string().color("blue")
                                    ));
                                }
                            } else {
                                break;
                            }
                        }
                        stack_strings.push(stack_str);
                    }
                    let stack_num = format!("{}", stack_strings.len());
                    if self.pieces[pt as usize][RED as usize] & (1u128 << index) != 0 {
                        if !is_on_stack {
                            res_str.push_str(&format!(" {} ", field_type.to_string().color("red")));
                        } else {
                            res_str.push_str(&format!(
                                " {}{}",
                                field_type.to_string().color("red"),
                                stack_num
                            ));
                        }
                    } else {
                        if !is_on_stack {
                            res_str
                                .push_str(&format!(" {} ", field_type.to_string().color("blue")));
                        } else {
                            res_str.push_str(&format!(
                                " {}{}",
                                field_type.to_string().color("blue"),
                                stack_num
                            ));
                        }
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
        for (i, stack_str) in stack_strings.iter().enumerate() {
            res_str.push_str(&format!("Stack {}: {}\n", i + 1, stack_str));
        }
        res_str.push_str(&format!(
            "Ply: {}\nColor to move: {:?}",
            self.ply, self.color_to_move
        ));
        write!(f, "{}", res_str)
    }
}
