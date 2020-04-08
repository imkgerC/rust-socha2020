extern crate game_sdk;
use game_sdk::neighbor_magic::generate_magic;
use game_sdk::{Action, GameState, HashKeys, PieceType};
use std::time::Instant;

fn main() {
    // Do some perft testing
    //FEN of midgame position:
    //FEN: 20 RED 19807040637789456435240771584 576460752303423488 0 0 0 140737488355328 0 0 0 34359738368 4835703278458585418301440 140737488355328 2417851639229258349412352 8388608 2361183241434822606848 144115188109410304 0 288230376151711744 576460752303423488
    //generate_magic();
    let state = GameState::from_fen("20 RED 19807040637789456435240771584 576460752303423488 0 0 0 140737488355328 0 0 0 34359738368 4835703278458585418301440 140737488355328 2417851639229258349412352 8388608 2361183241434822606848 144115188109410304 0 288230376151711744 576460752303423488".to_owned());
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
}
