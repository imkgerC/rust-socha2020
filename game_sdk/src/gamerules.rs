use crate::actionlist::ActionList;
use crate::bitboard;
use crate::gamestate::GameState;
use crate::piece_type::PieceType;

pub fn calculate_legal_moves(game_state: &GameState, actionlist: &mut ActionList) {
    if game_state.ply == 0 {
        // only SetMoves for every piece_type on every field
        return;
    }

    if game_state.ply == 1 {
        // only SetMoves next to only set enemy piece
        return;
    }

    if game_state.ply % 2 == 0 {
        // check if game is over
    }

    if game_state.must_player_place_bee() {
        // only bee SetMoves
        return;
    }

    // generate SetMoves

    if game_state.has_player_placed_bee() {
        // generate DragMoves
    }

    if actionlist.size == 0 {
        // add SkipMove to actionList
    }
}

pub fn is_game_finished(game_state: &GameState) -> bool {
    debug_assert!(game_state.ply % 2 == 0);

    if game_state.ply > 60 {
        return true;
    }

    if bitboard::get_neighbours(game_state.pieces[PieceType::BEE as usize][0]).count_ones() == 6 {
        return true;
    }

    if bitboard::get_neighbours(game_state.pieces[PieceType::BEE as usize][1]).count_ones() == 6 {
        return true;
    }

    return false;
}
