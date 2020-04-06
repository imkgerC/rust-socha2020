extern crate game_sdk;
use game_sdk::{Action, GameState, PieceType};
use std::time::Instant;

fn main() {
    // Do some perft testing
    let mut state = GameState::new();
    state.obstacles = (1 << 55) | (1 << 63) | (1 << 50);
    let now = Instant::now();
    let nodes = state.perft(3);
    let time_elapsed = now.elapsed().as_micros();
    let nps = (1000 * nodes) as f64 / time_elapsed as f64;
    println!("Time: {}, Nodes: {}, KNPS: {}", time_elapsed, nodes, nps);
    println!("{}", state);
}
