use game_sdk::{actionlist::ActionList, gamerules, Action, GameState};
use rand::{rngs::SmallRng, RngCore};

pub fn playout(initial: &GameState, al: &mut ActionList<Action>, rng: &mut SmallRng) -> f32 {
    let mut state = initial.clone();

    while !gamerules::is_game_finished(&state) {
        gamerules::calculate_legal_moves(&state, al);
        let rand = rng.next_u64() as usize % al.size;
        let action = al[rand];
        state.make_action(action);
    }
    if let Some(winner) = gamerules::get_result(&state) {
        if winner == initial.color_to_move {
            0.0
        } else {
            1.0
        }
    } else {
        0.5
    }
}
