mod graph;
mod playout;
mod rave_table;

use crate::search::Searcher;
use crate::timecontrol::Timecontrol;
use game_sdk::{Action, ActionList, ClientListener, GameState};
use graph::Node;
use hashbrown::HashMap;
use rand::{rngs::SmallRng, SeedableRng};
use rave_table::RaveTable;
use std::time::Instant;

pub struct MCTS {
    pub iterations_per_ms: f64,
    pub root: Node,
    pub tc: Timecontrol,
    rave_table: RaveTable,
    initial_state: GameState,
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
            rave_table: RaveTable::new(),
            initial_state: GameState::new(),
        }
    }

    pub fn search_nodes(&mut self, state: &GameState, n: usize, rng: &mut SmallRng) {
        let mut al = ActionList::default();
        for _ in 0..n {
            self.root
                .iteration(&mut state.clone(), &mut self.rave_table, &mut al, rng, true);
        }
    }

    fn set_root(&mut self, state: &GameState) {
        // assumes that next state is always exactly two ply away
        let mut first_index = None;
        let mut second_index = None;
        for (edge_idx, edge) in self.root.children.iter().enumerate() {
            self.initial_state.make_action(edge.action);
            for (inner_idx, inner_edge) in edge.node.children.iter().enumerate() {
                self.initial_state.make_action(inner_edge.action);
                if self.initial_state == *state {
                    first_index = Some(edge_idx);
                    second_index = Some(inner_idx);
                    break;
                }
                self.initial_state.unmake_action(inner_edge.action);
            }
            self.initial_state.unmake_action(edge.action);
        }
        self.initial_state = state.clone();
        if let Some(first_index) = first_index {
            if let Some(second_index) = second_index {
                self.root = self
                    .root
                    .children
                    .remove(first_index)
                    .node
                    .children
                    .remove(second_index)
                    .node;
                return;
            }
        }
        self.root = Node::empty();
    }

    pub fn search(&mut self, state: &GameState) {
        let start_time = Instant::now();
        println!("Searching state w/ fen:{}", state.to_fen());

        // tree reuse if possible
        self.set_root(state);

        let mut rng = SmallRng::from_entropy();
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
            let (score, pv_nodes, pv_move) = self.root.best_action();
            println!(
                "info depth {} score {} bestmove {:?} nodes {} nps {:.2} pvnodes {} time {} pv {}",
                pv_depth,
                (score * 100.) as isize,
                pv_move,
                self.root.n as usize,
                self.iterations_per_ms * 1000.,
                pv_nodes,
                elapsed,
                Searcher::format_pv(&pv)
            );
        }
        let (score, _, pv_move) = self.root.best_action();
        println!(
            "Finished search with move {:?} and score {}, pv: {}",
            pv_move,
            score as usize,
            Searcher::format_pv(&pv)
        );
    }

    pub fn best_action(&self) -> Action {
        self.root.best_action().2
    }
}

impl ClientListener for MCTS {
    fn on_move_request(&mut self, state: &GameState) -> Action {
        self.search(state);
        self.best_action()
    }
}
