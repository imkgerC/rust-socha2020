use game_sdk::{ClientListener, GameState};
use player::search::Searcher;
use player::timecontrol::Timecontrol;
use std::io;

fn main() {
    let mut searcher = Searcher::with_tc(Timecontrol::MoveTime(500));
    let stdin = io::stdin();
    let mut line = String::new();
    loop {
        line.clear();
        stdin.read_line(&mut line).ok().unwrap();
        let arg: Vec<&str> = line.split_whitespace().collect();
        let cmd = arg[0];
        match cmd {
            "exit" | "quit" => break,
            "requestmove" | "moverequest" => {
                let fen = arg[1..].join(" ");
                let state = GameState::from_fen(fen);
                let action = searcher.on_move_request(&state);
                println!("bestmove {}", action.to_string());
            }
            _ => continue,
        }
    }
}
