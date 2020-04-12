use crate::action::Action;
use crate::actionlist::ActionList;
use crate::bitboard;
use crate::bitboard::get_neighbours;
use crate::gamestate::Color;
use crate::gamestate::Color::{BLUE, RED};
use crate::gamestate::GameState;
use crate::neighbor_magic::get_accessible_neighbors;
use crate::piece_type::PieceType;

impl GameState {
    #[inline(always)]
    pub fn must_player_place_bee(&self) -> bool {
        let round = self.ply / 2;
        if round == 3 {
            if !self.has_player_placed_bee() {
                return true;
            }
        }
        return false;
    }

    #[inline(always)]
    pub fn has_player_placed_bee(&self) -> bool {
        return self.pieces[PieceType::BEE as usize][self.color_to_move as usize] > 0;
    }

    #[inline(always)]
    pub fn is_on_colored_stack(&self, index: usize, color: Color) -> bool {
        (1u128 << index) & self.beetle_stack[0][color as usize] != 0
    }

    #[inline(always)]
    pub fn is_on_stack(&self, index: usize) -> bool {
        self.is_on_colored_stack(index, Color::RED) || self.is_on_colored_stack(index, Color::BLUE)
    }

    #[inline(always)]
    pub fn occupied(&self) -> u128 {
        self.occupied[Color::RED as usize] | self.occupied[Color::BLUE as usize]
    }

    #[inline(always)]
    pub fn valid_set_destinations(&self, color: Color) -> u128 {
        let next_to_own = bitboard::get_neighbours(self.occupied[color as usize]);
        let next_to_other = bitboard::get_neighbours(self.occupied[color.swap() as usize]);
        next_to_own & !(next_to_other | self.obstacles | self.occupied())
    }
}
pub fn calculate_legal_moves(game_state: &GameState, actionlist: &mut ActionList<Action>) {
    debug_assert!(game_state.check_integrity());
    actionlist.size = 0;
    if game_state.ply == 0 {
        // SetMoves for every field and every PieceType
        let mut valid_fields = bitboard::constants::VALID_FIELDS & !game_state.obstacles;
        while valid_fields > 0 {
            let to = valid_fields.trailing_zeros();
            valid_fields ^= 1 << to;
            for piece_type in &crate::piece_type::PIECETYPE_VARIANTS {
                actionlist.push(Action::SetMove(*piece_type, to as u8));
            }
        }
        return;
    }

    if game_state.ply == 1 {
        // only SetMoves next to only set enemy piece
        // enemy is always red in first move
        let next_to_enemy = bitboard::get_neighbours(game_state.occupied[Color::RED as usize]);
        let mut valid_fields = next_to_enemy & !game_state.obstacles;
        while valid_fields > 0 {
            let to = valid_fields.trailing_zeros();
            valid_fields ^= 1 << to;
            for piece_type in &crate::piece_type::PIECETYPE_VARIANTS {
                actionlist.push(Action::SetMove(*piece_type, to as u8));
            }
        }
        return;
    }
    let mut valid_set_destinations = game_state.valid_set_destinations(game_state.color_to_move);

    if game_state.must_player_place_bee() {
        // only bee SetMoves
        while valid_set_destinations > 0 {
            let to = valid_set_destinations.trailing_zeros();
            valid_set_destinations ^= 1 << to;
            actionlist.push(Action::SetMove(PieceType::BEE, to as u8));
        }
        if actionlist.size == 0 {
            actionlist.push(Action::SkipMove);
        }
        return;
    }

    // generate SetMoves
    let undeployed_counts = game_state.undeployed_counts[game_state.color_to_move as usize];
    while valid_set_destinations > 0 {
        let to = valid_set_destinations.trailing_zeros();
        valid_set_destinations ^= 1 << to;
        for piece_type in &crate::piece_type::PIECETYPE_VARIANTS {
            if undeployed_counts[*piece_type as usize] > 0 {
                actionlist.push(Action::SetMove(*piece_type, to as u8));
            }
        }
    }

    if game_state.has_player_placed_bee() {
        // generate DragMoves
        calculate_drag_moves(game_state, actionlist);
    }

    if actionlist.size == 0 {
        // add SkipMove to actionList
        actionlist.push(Action::SkipMove);
    }
}

fn calculate_drag_moves(game_state: &GameState, actionlist: &mut ActionList<Action>) {
    let mut own_fields = game_state.occupied[game_state.color_to_move as usize];
    while own_fields > 0 {
        let from = own_fields.trailing_zeros() as u8;
        let from_bit = 1 << from;
        own_fields ^= from_bit;
        if from_bit
            & (game_state.beetle_stack[0][Color::RED as usize]
                | game_state.beetle_stack[0][Color::BLUE as usize])
            > 0
        {
            // beetle move generation does not need to check swarm connected-ness if beetle is on top of sth
            // and accessibility is easy as well
            let mut valid_destinations = bitboard::get_neighbours(from_bit) & !game_state.obstacles;
            while valid_destinations > 0 {
                let to = valid_destinations.trailing_zeros() as u8;
                valid_destinations ^= 1 << to;
                actionlist.push(Action::DragMove(PieceType::BEETLE, from, to));
            }
            continue;
        }

        // check if field can be removed and swarm is still connected
        let occupied = (game_state.occupied[Color::RED as usize]
            | game_state.occupied[Color::BLUE as usize])
            ^ from_bit;
        let neighbours = bitboard::get_neighbours(from_bit) & occupied;
        if !are_connected_in_swarm(occupied, neighbours) {
            continue;
        }
        if from_bit & game_state.pieces[PieceType::BEE as usize][game_state.color_to_move as usize]
            > 0
        {
            // bee move generation
            let mut valid = get_accessible_neighbors(occupied, game_state.obstacles, from_bit);
            while valid > 0 {
                let to = valid.trailing_zeros() as u8;
                valid ^= 1 << to;
                actionlist.push(Action::DragMove(PieceType::BEE, from, to));
            }
            continue;
        }
        if from_bit
            & game_state.pieces[PieceType::BEETLE as usize][game_state.color_to_move as usize]
            > 0
        {
            // beetle move generation
            let mut valid =
                get_beetle_accessible_neighbours(occupied, game_state.obstacles, from_bit);
            while valid > 0 {
                let to = valid.trailing_zeros() as u8;
                valid ^= 1 << to;
                actionlist.push(Action::DragMove(PieceType::BEETLE, from, to));
            }
            continue;
        }
        if from_bit & game_state.pieces[PieceType::ANT as usize][game_state.color_to_move as usize]
            > 0
        {
            // ant move generation
            let mut valid = get_ant_destinations(occupied, game_state.obstacles, from_bit);
            while valid > 0 {
                let to = valid.trailing_zeros() as u8;
                valid ^= 1 << to;
                actionlist.push(Action::DragMove(PieceType::ANT, from, to));
            }
            continue;
        }
        if from_bit
            & game_state.pieces[PieceType::SPIDER as usize][game_state.color_to_move as usize]
            > 0
        {
            // spider move generation
            let mut valid = 0;
            append_spider_destinations(
                &mut valid,
                occupied,
                game_state.obstacles,
                from_bit,
                from_bit,
                3,
            );
            while valid > 0 {
                let to = valid.trailing_zeros() as u8;
                valid ^= 1 << to;
                actionlist.push(Action::DragMove(PieceType::SPIDER, from, to));
            }
            continue;
        }
        if from_bit
            & game_state.pieces[PieceType::GRASSHOPPER as usize][game_state.color_to_move as usize]
            > 0
        {
            // grasshopper move generation
            let mut valid = get_grasshopper_destinations(occupied, game_state.obstacles, from_bit);
            while valid > 0 {
                let to = valid.trailing_zeros() as u8;
                valid ^= 1 << to;
                actionlist.push(Action::DragMove(PieceType::GRASSHOPPER, from, to));
            }
            continue;
        }
    }
}

fn get_grasshopper_destinations(occupied: u128, obstacles: u128, from: u128) -> u128 {
    let mut destinations = 0;

    // nowe
    let mut nowe = bitboard::shift_nowe(from);
    while nowe & occupied > 0 {
        nowe = bitboard::shift_nowe(nowe);
    }
    destinations |= nowe;

    // noea
    let mut noea = bitboard::shift_noea(from);
    while noea & occupied > 0 {
        noea = bitboard::shift_noea(noea);
    }
    destinations |= noea;

    // soea
    let mut soea = bitboard::shift_soea(from);
    while soea & occupied > 0 {
        soea = bitboard::shift_soea(soea);
    }
    destinations |= soea;

    // sowe
    let mut sowe = bitboard::shift_sowe(from);
    while sowe & occupied > 0 {
        sowe = bitboard::shift_sowe(sowe);
    }
    destinations |= sowe;

    // east
    let mut east = bitboard::shift_east(from);
    while east & occupied > 0 {
        east = bitboard::shift_east(east);
    }
    destinations |= east;

    // west
    let mut west = bitboard::shift_west(from);
    while west & occupied > 0 {
        west = bitboard::shift_west(west);
    }
    destinations |= west;

    return destinations & !(obstacles | bitboard::get_neighbours(from));
}

fn get_ant_destinations(occupied: u128, obstacles: u128, current_field: u128) -> u128 {
    let mut candidates = get_accessible_neighbors(occupied, obstacles, current_field);
    let mut destinations = candidates;
    while candidates > 0 {
        let current = candidates.trailing_zeros();
        let current_field = 1 << current;
        candidates ^= current_field;
        candidates |= get_accessible_neighbors(occupied, obstacles, current_field) & !destinations;
        destinations |= candidates;
    }
    return destinations & !current_field;
}

fn append_spider_destinations(
    destinations: &mut u128,
    occupied: u128,
    obstacles: u128,
    current_field: u128,
    current_path: u128,
    to_go: u8,
) {
    let mut candidates = get_accessible_neighbors(occupied, obstacles, current_field);
    candidates &= !current_path;
    if to_go == 1 {
        *destinations |= candidates;
        return;
    }
    while candidates > 0 {
        let current = candidates.trailing_zeros();
        let current_field = 1 << current;
        candidates ^= current_field;
        append_spider_destinations(
            destinations,
            occupied,
            obstacles,
            current_field,
            current_path ^ current_field,
            to_go - 1,
        );
    }
}

fn get_beetle_accessible_neighbours(occupied: u128, obstacles: u128, field: u128) -> u128 {
    let mut ret = 0;
    let nowe = bitboard::shift_nowe(field);
    let noea = bitboard::shift_noea(field);
    let sowe = bitboard::shift_sowe(field);
    let soea = bitboard::shift_soea(field);
    let east = bitboard::shift_east(field);
    let west = bitboard::shift_west(field);
    // check nowe
    let nowe_check = west | noea | nowe;
    if nowe_check & occupied > 0 {
        ret |= nowe;
    }
    // check west
    let west_check = nowe | sowe | west;
    if west_check & occupied > 0 {
        ret |= west;
    }
    // check noea
    let noea_check = nowe | east | nowe;
    if noea_check & occupied > 0 {
        ret |= noea;
    }
    // check east
    let east_check = noea | soea | east;
    if east_check & occupied > 0 {
        ret |= east;
    }
    // check sowe
    let sowe_check = soea | west | sowe;
    if sowe_check & occupied > 0 {
        ret |= sowe;
    }
    // check soea
    let soea_check = east | sowe | soea;
    if soea_check & occupied > 0 {
        ret |= soea;
    }

    return ret & !obstacles;
}

pub fn are_connected_in_swarm(occupied: u128, to_check: u128) -> bool {
    if to_check.count_ones() == 1 {
        return true;
    }
    let mut visited = 1u128 << to_check.trailing_zeros();
    let mut old_visited = 0;
    while visited != old_visited {
        old_visited = visited;
        visited |= bitboard::get_neighbours(visited) & occupied;
        if visited & to_check == to_check {
            return true;
        }
    }
    return false;
}

//Only works if is_game_finished is true
pub fn get_result(game_state: &GameState) -> Option<Color> {
    if game_state.pieces[PieceType::BEE as usize][RED as usize] == 0 {
        Some(BLUE)
    } else if game_state.pieces[PieceType::BEE as usize][BLUE as usize] == 0 {
        Some(RED)
    } else {
        let red_neighbor_count =
            (get_neighbours(game_state.pieces[PieceType::BEE as usize][RED as usize])
                & !game_state.occupied()
                & !game_state.obstacles)
                .count_ones();
        let blue_neighbor_count =
            (get_neighbours(game_state.pieces[PieceType::BEE as usize][BLUE as usize])
                & !game_state.occupied()
                & !game_state.obstacles)
                .count_ones();
        if red_neighbor_count > blue_neighbor_count {
            Some(RED)
        } else if blue_neighbor_count > red_neighbor_count {
            Some(BLUE)
        } else {
            None
        }
    }
}

pub fn is_game_finished(game_state: &GameState) -> bool {
    if game_state.ply >= 60 {
        return true;
    }

    if game_state.ply % 2 == 1 {
        return false;
    }

    if game_state.pieces[PieceType::BEE as usize][Color::RED as usize] != 0 {
        let bee_neighbours = bitboard::get_neighbours(
            game_state.pieces[PieceType::BEE as usize][Color::RED as usize],
        );
        if (bee_neighbours
            & (game_state.occupied[Color::BLUE as usize]
                | game_state.occupied[Color::RED as usize]
                | game_state.obstacles))
            == bee_neighbours
        {
            return true;
        }
    } else if game_state.ply >= 7 {
        return true;
    }
    if game_state.pieces[PieceType::BEE as usize][Color::BLUE as usize] != 0 {
        let bee_neighbours = bitboard::get_neighbours(
            game_state.pieces[PieceType::BEE as usize][Color::BLUE as usize],
        );
        if (bee_neighbours
            & (game_state.occupied[Color::BLUE as usize]
                | game_state.occupied[Color::RED as usize]
                | game_state.obstacles))
            == bee_neighbours
        {
            return true;
        }
    } else if game_state.ply >= 8 {
        return true;
    }

    return false;
}
