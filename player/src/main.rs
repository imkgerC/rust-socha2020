extern crate game_sdk;
use game_sdk::{Action, GameState, HashKeys, PieceType};
use std::time::Instant;

fn main() {
    // Do some perft testing
    let mut state = GameState::new();
    let now = Instant::now();
    let nodes = state.perft(6);
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
