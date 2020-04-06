extern crate game_sdk;
use game_sdk::GameState;
use std::time::Instant;

fn main() {
    // Do some perft testing
    let state = GameState::new();
    let now = Instant::now();
    let nodes = state.perft(1);
    let time_elapsed = now.elapsed().as_micros();
    let nps = (1000 * nodes) as f64 / time_elapsed as f64;
    println!("Time: {} ,Nodes: {}, KNPS: {}", time_elapsed, nodes, nps);
}
