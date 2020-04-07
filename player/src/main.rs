extern crate game_sdk;
use game_sdk::neighbor_magic::generate_magic;
use game_sdk::{Action, GameState, HashKeys, PieceType};
use std::time::Instant;

fn main() {
    // Do some perft testing
    generate_magic();
    let mut state = GameState::new();
    state.obstacles |= (1u128 << 13) | (1u128 << 63) | (1u128 << 94);
    state.make_action(Action::SetMove(PieceType::BEETLE, 47));
    state.make_action(Action::SetMove(PieceType::SPIDER, 59));
    state.make_action(Action::SetMove(PieceType::BEETLE, 46));
    state.make_action(Action::SetMove(PieceType::BEETLE, 70));
    state.make_action(Action::SetMove(PieceType::BEE, 35));
    state.make_action(Action::SetMove(PieceType::ANT, 60));
    state.make_action(Action::SetMove(PieceType::ANT, 36));
    state.make_action(Action::SetMove(PieceType::BEE, 71));
    state.make_action(Action::SetMove(PieceType::ANT, 57));
    state.make_action(Action::DragMove(PieceType::ANT, 60, 25));
    state.make_action(Action::SetMove(PieceType::SPIDER, 23));
    state.make_action(Action::SetMove(PieceType::GRASSHOPPER, 60));
    state.make_action(Action::DragMove(PieceType::ANT, 57, 82));
    state.make_action(Action::DragMove(PieceType::BEETLE, 70, 59));
    state.make_action(Action::DragMove(PieceType::BEETLE, 46, 58));
    state.make_action(Action::DragMove(PieceType::BEETLE, 59, 47));
    state.make_action(Action::DragMove(PieceType::BEETLE, 58, 59));
    state.make_action(Action::DragMove(PieceType::GRASSHOPPER, 60, 58));
    state.make_action(Action::SetMove(PieceType::GRASSHOPPER, 81));
    state.make_action(Action::SetMove(PieceType::ANT, 57));
    println!("{}", state);
    let now = Instant::now();
    let nodes = state.perft(3);
    let time_elapsed = now.elapsed().as_micros();
    let nps = (1000 * nodes) as f64 / time_elapsed as f64;
    println!(
        "Time: {}ms, Nodes: {}, KNPS: {}",
        time_elapsed as f64 / 1000.,
        nodes,
        nps
    );
    println!("{}", state);
}
