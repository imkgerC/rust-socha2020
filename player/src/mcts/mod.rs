mod graph;
mod playout;

use crate::search::Searcher;
use crate::timecontrol::Timecontrol;
use game_sdk::{Action, ActionList, ClientListener, GameState};
use graph::Node;
use rand::{rngs::SmallRng, SeedableRng};
use std::time::Instant;

pub struct MCTS {
    pub iterations_per_ms: f64,
    pub root: Node,
    pub tc: Timecontrol,
}

impl MCTS {
    pub fn new() -> Self {
        MCTS::with_tc(Timecontrol::MoveTime(1700))
    }
    pub fn with_tc(tc: Timecontrol) -> Self {
        MCTS {
            iterations_per_ms: 0.5,
            root: Node::empty(),
            tc,
        }
    }

    pub fn search_nodes(&mut self, state: &GameState, n: usize, rng: &mut SmallRng) {
        let mut al = ActionList::default();
        for _ in 0..n {
            self.root.iteration(&mut state.clone(), &mut al, rng);
        }
    }

    pub fn search(&mut self, state: &GameState) {
        let start_time = Instant::now();
        println!("Searching state w/ fen:{}", state.to_fen());
        let mut rng = SmallRng::from_entropy();
        self.root = Node::empty();
        let mut samples = 0;
        let mut elapsed = 0;
        self.iterations_per_ms = 1.;
        let mut pv = ActionList::default();
        loop {
            let time_left = self.tc.time_left(elapsed);
            if time_left < 50 {
                break;
            }
            let to_search = ((time_left as f64 / 2.) * self.iterations_per_ms).max(1.) as usize;
            self.search_nodes(state, to_search, &mut rng);
            samples += to_search;
            pv.clear();
            let pv_depth = self.root.build_pv(&mut state.clone(), &mut pv);
            elapsed = start_time.elapsed().as_millis() as u64;
            self.iterations_per_ms = samples as f64 / elapsed as f64;
            let (score, pv_move) = self.root.best_action();
            println!(
                "info depth {} score {} bestmove {:?} nodes {} nps {:.2} time {} pv {}",
                pv_depth,
                score as usize,
                pv_move,
                samples,
                self.iterations_per_ms * 1000.,
                elapsed,
                Searcher::format_pv(&pv)
            );
        }
        let (score, pv_move) = self.root.best_action();
        println!(
            "Finished search with move {:?} and score {}, pv: {}",
            pv_move,
            score as usize,
            Searcher::format_pv(&pv)
        );
    }

    pub fn best_action(&self) -> Action {
        self.root.best_action().1
    }
}

impl ClientListener for MCTS {
    fn on_move_request(&mut self, state: &GameState) -> Action {
        self.search(state);
        self.best_action()
    }
}
