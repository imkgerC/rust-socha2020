use crate::action::Action;
use crate::actionlist::ActionList;
use crate::bitboard;
use crate::gamestate::Color;
use crate::gamestate::GameState;
use crate::piece_type::PieceType;

pub fn calculate_legal_moves(game_state: &GameState, actionlist: &mut ActionList) {
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
    while valid_set_destinations > 0 {
        let to = valid_set_destinations.trailing_zeros();
        valid_set_destinations ^= 1 << to;
        for piece_type in &crate::piece_type::VARIANTS {
            actionlist.push(Action::SetMove(*piece_type, to as u8));
        }
    }

    if game_state.has_player_placed_bee() {
        // generate DragMoves
    }

    if actionlist.size == 0 {
        // add SkipMove to actionList
        actionlist.push(Action::SkipMove);
    }
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
