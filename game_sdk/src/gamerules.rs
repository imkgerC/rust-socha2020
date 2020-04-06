use crate::action::Action;
use crate::actionlist::ActionList;
use crate::bitboard;
use crate::gamestate::Color;
use crate::gamestate::GameState;
use crate::piece_type::PieceType;

pub fn calculate_legal_moves(game_state: &GameState, actionlist: &mut ActionList) {
    actionlist.size = 0;
    if game_state.ply == 0 {
        // SetMoves for every field and every PieceType
        let mut valid_fields = bitboard::constants::VALID_FIELDS & !game_state.obstacles;
        while valid_fields > 0 {
            let to = valid_fields.trailing_zeros();
            valid_fields ^= 1 << to;
            for piece_type in &crate::piece_type::VARIANTS {
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
            for piece_type in &crate::piece_type::VARIANTS {
                actionlist.push(Action::SetMove(*piece_type, to as u8));
            }
        }
        return;
    }

    let next_to_own =
        bitboard::get_neighbours(game_state.occupied[game_state.color_to_move as usize]);
    let next_to_other =
        bitboard::get_neighbours(game_state.occupied[game_state.color_to_move.swap() as usize]);
    let mut valid_set_destinations = next_to_own & !(next_to_other | game_state.obstacles);

    if game_state.must_player_place_bee() {
        // only bee SetMoves
        while valid_set_destinations > 0 {
            let to = valid_set_destinations.trailing_zeros();
            valid_set_destinations ^= 1 << to;
            actionlist.push(Action::SetMove(PieceType::BEE, to as u8));
        }
        return;
    }

    // generate SetMoves
    let mut allowed = Vec::with_capacity(5);
    if game_state.pieces[PieceType::BEE as usize][game_state.color_to_move as usize] == 0 {
        allowed.push(PieceType::BEE);
    }
    if game_state.pieces[PieceType::ANT as usize][game_state.color_to_move as usize].count_ones()
        < 3
    {
        allowed.push(PieceType::ANT);
    }
    if game_state.pieces[PieceType::SPIDER as usize][game_state.color_to_move as usize].count_ones()
        < 3
    {
        allowed.push(PieceType::SPIDER);
    }
    if game_state.pieces[PieceType::GRASSHOPPER as usize][game_state.color_to_move as usize]
        .count_ones()
        < 2
    {
        allowed.push(PieceType::GRASSHOPPER);
    }
    if game_state.pieces[PieceType::BEETLE as usize][game_state.color_to_move as usize].count_ones()
        < 2
    {
        allowed.push(PieceType::BEETLE);
    }
    while valid_set_destinations > 0 {
        let to = valid_set_destinations.trailing_zeros();
        valid_set_destinations ^= 1 << to;
        // TODO: check which pieces were already set
        for piece_type in &allowed {
            actionlist.push(Action::SetMove(*piece_type, to as u8));
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

fn calculate_drag_moves(game_state: &GameState, actionlist: &mut ActionList) {
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
        if !are_connected_in_swarm(game_state, occupied, neighbours) {
            continue;
        }
        if from_bit & game_state.pieces[PieceType::BEE as usize][game_state.color_to_move as usize]
            > 0
        {
            // bee move generation
            let mut valid = get_accessible_neighbours(occupied, game_state.obstacles, from_bit);
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
    let mut candidates = get_accessible_neighbours(occupied, obstacles, current_field);
    let mut destinations = candidates;
    while candidates > 0 {
        let current = candidates.trailing_zeros();
        let current_field = 1 << current;
        candidates ^= current_field;
        candidates |= get_accessible_neighbours(occupied, obstacles, current_field);
        candidates &= !destinations;
        destinations |= candidates;
    }
    return destinations & !current_field;
}

fn append_spider_destinations(
    destinations: &mut u128,
    occupied: u128,
    obstacles: u128,
    current_field: u128,
    mut current_path: u128,
    to_go: u8,
) {
    let mut candidates = get_accessible_neighbours(occupied, obstacles, current_field);
    candidates &= !current_path;
    if to_go == 1 {
        *destinations |= candidates;
        return;
    }
    while candidates > 0 {
        let current = candidates.trailing_zeros();
        let current_field = 1 << current;
        candidates ^= current_field;
        current_path ^= current_field;
        append_spider_destinations(
            destinations,
            occupied,
            obstacles,
            current_field,
            current_path,
            to_go - 1,
        );
        current_path ^= current_field;
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
    // check east
    let east_check = east | sowe | east;
    if east_check & occupied > 0 {
        ret |= east;
    }

    return ret & !obstacles;
}

fn get_accessible_neighbours(occupied: u128, obstacles: u128, field: u128) -> u128 {
    let free = !(occupied | obstacles);
    let mut ret = 0;
    let nowe = bitboard::shift_nowe(field);
    let noea = bitboard::shift_noea(field);
    let sowe = bitboard::shift_sowe(field);
    let soea = bitboard::shift_soea(field);
    let east = bitboard::shift_east(field);
    let west = bitboard::shift_west(field);
    // check nowe
    let nowe_check = west | noea;
    if nowe_check.count_ones() == 1 {
        if nowe_check & occupied > 0 {
            ret |= nowe;
        }
    } else {
        if nowe_check & occupied > 0 && nowe_check & free > 0 {
            ret |= nowe;
        }
    }
    // check west
    let west_check = nowe | sowe;
    if west_check.count_ones() == 1 {
        if west_check & occupied > 0 {
            ret |= west;
        }
    } else {
        if west_check & occupied > 0 && west_check & free > 0 {
            ret |= west;
        }
    }
    // check noea
    let noea_check = nowe | east;
    if noea_check.count_ones() == 1 {
        if noea_check & occupied > 0 {
            ret |= noea;
        }
    } else {
        if noea_check & occupied > 0 && noea_check & free > 0 {
            ret |= noea;
        }
    }
    // check east
    let east_check = noea | soea;
    if east_check.count_ones() == 1 {
        if east_check & occupied > 0 {
            ret |= east;
        }
    } else {
        if east_check & occupied > 0 && east_check & free > 0 {
            ret |= east;
        }
    }
    // check sowe
    let sowe_check = soea | west;
    if sowe_check.count_ones() == 1 {
        if sowe_check & occupied > 0 {
            ret |= sowe;
        }
    } else {
        if sowe_check & occupied > 0 && sowe_check & free > 0 {
            ret |= sowe;
        }
    }
    // check east
    let east_check = soea | noea;
    if east_check.count_ones() == 1 {
        if east_check & occupied > 0 {
            ret |= east;
        }
    } else {
        if east_check & occupied > 0 && east_check & free > 0 {
            ret |= east;
        }
    }

    return ret & free;
}

fn are_connected_in_swarm(game_state: &GameState, occupied: u128, to_check: u128) -> bool {
    if to_check == 0 {
        println!("{}", game_state);
        println!("{}", occupied);
        panic!("is not allowed by the rules");
    }
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

pub fn is_game_finished(game_state: &GameState) -> bool {
    if game_state.ply > 60 {
        return true;
    }

    if game_state.ply % 2 == 1 {
        return false;
    }

    let bee_neighbours =
        bitboard::get_neighbours(game_state.pieces[PieceType::BEE as usize][Color::RED as usize]);
    if (bee_neighbours
        & (game_state.occupied[Color::BLUE as usize]
            | game_state.occupied[Color::RED as usize]
            | game_state.obstacles))
        .count_ones()
        == 6
    {
        return true;
    }
    let bee_neighbours =
        bitboard::get_neighbours(game_state.pieces[PieceType::BEE as usize][Color::BLUE as usize]);
    if (bee_neighbours
        & (game_state.occupied[Color::BLUE as usize]
            | game_state.occupied[Color::RED as usize]
            | game_state.obstacles))
        .count_ones()
        == 6
    {
        return true;
    }

    return false;
}
