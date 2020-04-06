use crate::actionlist::ActionList;
use crate::gamestate::GameState;

pub fn caculate_legal_moves(game_state: GameState, actionlist: &mut ActionList) {
    if game_state.ply == 0 {
        // only SetMoves for every piece_type on every field
        return;
    }

    if game_state.ply == 1 {
        // only SetMoves next to only set enemy piece
        return;
    }

    if game_state.must_player_set_bee() {
        // only bee SetMoves
        return;
    }

    // generate SetMoves

    if game_state.has_player_placed_bee() {
        // generate DragMoves
    }
}
