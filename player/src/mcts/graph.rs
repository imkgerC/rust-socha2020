use super::playout::playout;
use game_sdk::{gamerules, Action, ActionList, GameState};

const C: f32 = 0.5;
const C_BASE: f32 = 7000.;
const C_FACTOR: f32 = 0.5;

pub struct Node {
    pub n: f32,
    pub q: f32,
    children: Vec<Edge>,
}

impl Node {
    pub fn empty() -> Self {
        Node {
            n: 0.,
            q: 0.,
            children: Vec::new(),
        }
    }

    pub fn iteration(&mut self, state: &mut GameState, al: &mut ActionList<Action>) -> f32 {
        let delta;
        let c_adjusted = C + C_FACTOR * ((1. + self.n + C_BASE) / C_BASE).ln();
        if self.children.len() == 0 {
            if !gamerules::is_game_finished(state) {
                gamerules::calculate_legal_moves(state, al);
                self.children = Vec::with_capacity(al.size);
                for i in 0..al.size {
                    self.children.push(Edge::new(al[i]));
                }
                delta = playout(state, al);
            } else if self.n == 0. {
                self.q = if let Some(winner) = gamerules::get_result(&state) {
                    if winner == state.color_to_move {
                        0.0
                    } else {
                        1.0
                    }
                } else {
                    0.5
                };
                self.n = 1.;
                delta = self.q / self.n;
            } else {
                delta = self.q / self.n;
            }
            self.backpropagate(delta);
            return 1. - delta;
        }
        let mut best_edge = 0;
        let mut best_value = std::f32::NEG_INFINITY;
        for (edge_idx, edge) in self.children.iter().enumerate() {
            let value = edge.get_uct_value(self.n, c_adjusted);
            if value >= best_value {
                best_edge = edge_idx;
                best_value = value;
            }
        }
        delta = self.children[best_edge].iteration(state, al);
        self.backpropagate(delta);
        return 1. - delta;
    }

    pub fn backpropagate(&mut self, q: f32) {
        self.q += q;
        self.n += 1.;
    }

    pub fn best_action(&self) -> (f32, Action) {
        if self.children.len() == 0 {
            panic!("no action in terminal state");
        }

        let mut best_edge = 0;
        let mut best_value = std::f32::NEG_INFINITY;
        for (edge_idx, edge) in self.children.iter().enumerate() {
            let value = edge.get_value();
            if value >= best_value {
                best_edge = edge_idx;
                best_value = value;
            }
        }
        return (best_value, self.children[best_edge].action);
    }

    pub fn build_pv(&self, state: &mut GameState, al: &mut ActionList<Action>) -> usize {
        if self.children.len() == 0 {
            return 0;
        }

        let mut best_edge = 0;
        let mut best_value = std::f32::NEG_INFINITY;
        for (edge_idx, edge) in self.children.iter().enumerate() {
            let value = edge.get_value();
            if value >= best_value {
                best_edge = edge_idx;
                best_value = value;
            }
        }
        return self.children[best_edge].build_pv(state, al) + 1;
    }
}

pub struct Edge {
    pub action: Action,
    node: Node,
}

impl Edge {
    pub fn new(action: Action) -> Self {
        Edge {
            action,
            node: Node::empty(),
        }
    }

    pub fn iteration(&mut self, state: &mut GameState, al: &mut ActionList<Action>) -> f32 {
        state.make_action(self.action);
        self.node.iteration(state, al)
    }

    pub fn get_uct_value(&self, parent_n: f32, c: f32) -> f32 {
        if self.node.n > 0. {
            (self.node.q / self.node.n) + c * (parent_n.ln() / self.node.n).sqrt()
        } else {
            std::f32::INFINITY
        }
    }

    pub fn get_value(&self) -> f32 {
        if self.node.n > 0. {
            self.node.q / self.node.n
        } else {
            std::f32::NEG_INFINITY
        }
    }

    pub fn build_pv(&self, state: &mut GameState, al: &mut ActionList<Action>) -> usize {
        al.push(self.action);
        self.node.build_pv(state, al)
    }
}
