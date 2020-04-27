use game_sdk::{actionlist::ActionList, gamerules, Action, Color, GameState, PieceType};
use rand::{rngs::SmallRng, RngCore};

pub fn playout(initial: &GameState, al: &mut ActionList<Action>, rng: &mut SmallRng) -> f32 {
    let mut state = initial.clone();

    while !gamerules::is_game_finished(&state) {
        gamerules::calculate_legal_moves(&state, al);
        let rand = rng.next_u64() as usize % al.size;
        let action = al[rand];
        state.make_action(action);
    }
    get_score(&state, initial.color_to_move)
}

// if we win 0 - rate at loss 1 - rate draw = 0.5
fn get_score(state: &GameState, color: Color) -> f32 {
    // assumes state is terminal
    let other = color.swap();
    if state.pieces[PieceType::BEE as usize][color as usize] == 0 {
        return 1.125;
    }
    if state.pieces[PieceType::BEE as usize][other as usize] == 0 {
        return -0.125;
    }
    let own_free =
        (game_sdk::bitboard::get_neighbours(state.pieces[PieceType::BEE as usize][color as usize])
            & !state.occupied()
            & !state.obstacles)
            .count_ones();
    let other_free =
        (game_sdk::bitboard::get_neighbours(state.pieces[PieceType::BEE as usize][other as usize])
            & !state.occupied()
            & !state.obstacles)
            .count_ones();
    if own_free == other_free {
        return 0.5;
    }
    let rate = ((own_free as isize - other_free as isize) as f32) / 40.; // own_free - other_free can be 5 at max and -5 at min, scale to [-.125, .125]
    if own_free > other_free {
        return 0. - rate;
    }
    // else other is bigger
    1. - rate
}
