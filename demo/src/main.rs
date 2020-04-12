use game_sdk::{Action, ClientListener, Color, GameState};
use player::search::Searcher;
use player::timecontrol::Timecontrol;
use std::time::Instant;

fn main() {
    // Do some perft testing
    //FEN of midgame position:
    //FEN: 20 RED 19807040637789456435240771584 576460752303423488 0 0 0 140737488355328 0 0 0 34359738368 4835703278458585418301440 140737488355328 2417851639229258349412352 8388608 2361183241434822606848 144115188109410304 0 288230376151711744 576460752303423488
    //generate_magic();
    //panic!("stop");
    let states= vec![
        "48 RED 158456325028530926987170021376 18446744073709551616 0 0 0 0 0 0 0 137438953472 79228162516579187802012385280 68719476736 577586652210266112 324518553733984590649807832350720 18446744073709551616 38685630839424520762163200 18898689303515435630592 1329227995784915874056728564887191552 9444732966289079795712",
    ];
    let mut searcher = Searcher::with_tc(Timecontrol::Infinite);
    for state in states {
        let state = GameState::from_fen(state.to_owned());
        searcher.on_move_request(&state);
    }
    panic!("Stop debug");
    let state = GameState::from_fen("20 RED 19807040637789456435240771584 576460752303423488 0 0 0 140737488355328 0 0 0 34359738368 4835703278458585418301440 140737488355328 2417851639229258349412352 8388608 2361183241434822606848 144115188109410304 0 288230376151711744 576460752303423488".to_owned());
    let now = Instant::now();
    let nodes = state.perft(5);
    let time_elapsed = now.elapsed().as_micros();
    let nps = (1000 * nodes) as f64 / time_elapsed as f64;
    println!(
        "Time: {}ms, Nodes: {}, KNPS: {}",
        time_elapsed as f64 / 1000.,
        nodes,
        nps
    );
    for i in 1..=5 {
        println!("{}: {}", i, state.perft(i));
    }
    let mut searcher = Searcher::with_tc(Timecontrol::Infinite);
    let state = GameState::from_fen("37 BLUE 288230651063173120 0 72057594037927936 0 0 72057594037927936 0 0 0 72057594037927936 35184376285184 1 140754668224512 103087603712 65536 8204 2 16 147573952589676412928".to_owned());
    searcher.search_move(&state);
}
