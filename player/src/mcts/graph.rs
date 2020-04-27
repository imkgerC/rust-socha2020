use super::playout::playout;
use game_sdk::{gamerules, Action, ActionList, GameState};
use hashbrown::HashMap;
use rand::rngs::SmallRng;

const C: f32 = 0.1;
const C_BASE: f32 = 700.;
const C_FACTOR: f32 = 0.5;
const B_SQUARED: f32 = 0.5;

pub struct Node {
    pub n: f32,
    pub q: f32,
    pub children: Vec<Edge>,
}

impl Node {
    pub fn empty() -> Self {
        Node {
            n: 0.,
            q: 0.,
            children: Vec::new(),
        }
    }

    pub fn iteration(
        &mut self,
        state: &mut GameState,
        rave_table: &mut HashMap<Action, (f32, f32)>,
        al: &mut ActionList<Action>,
        rng: &mut SmallRng,
    ) -> f32 {
        let delta;
        let c_adjusted = C + C_FACTOR * ((1. + self.n + C_BASE) / C_BASE).ln();
        let fpu_r = -0.261;
        let fpu_base = (self.n - self.q) / self.n - fpu_r;
        let b_squared = B_SQUARED;
        if self.children.len() == 0 {
            if !gamerules::is_game_finished(state) {
                gamerules::calculate_legal_moves(state, al);
                self.children = Vec::with_capacity(al.size);
                for i in 0..al.size {
                    self.children.push(Edge::new(al[i]));
                }
                delta = playout(state, &state.color_to_move, rave_table, al, rng);
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
            let value = edge.get_uct_value(self.n, c_adjusted, b_squared, rave_table, fpu_base);
            if value >= best_value {
                best_edge = edge_idx;
                best_value = value;
            }
        }
        delta = self.children[best_edge].iteration(state, rave_table, al, rng);
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
    pub node: Node,
}

impl Edge {
    pub fn new(action: Action) -> Self {
        Edge {
            action,
            node: Node::empty(),
        }
    }

    pub fn iteration(
        &mut self,
        state: &mut GameState,
        rave_table: &mut HashMap<Action, (f32, f32)>,
        al: &mut ActionList<Action>,
        rng: &mut SmallRng,
    ) -> f32 {
        state.make_action(self.action);
        self.node.iteration(state, rave_table, al, rng)
    }

    pub fn get_uct_value(
        &self,
        parent_n: f32,
        c: f32,
        b_squared: f32,
        rave_table: &HashMap<Action, (f32, f32)>,
        fpu_base: f32,
    ) -> f32 {
        if self.node.n > 0. {
            let (rave_q, rave_n) = rave_table.get(&self.action).unwrap_or(&(0., 0.));
            let (rave_q, rave_n) = (*rave_q, *rave_n);
            let beta =
                (rave_n / (rave_n + self.node.n + 4. * b_squared * rave_n * self.node.n)).min(1.0);
            let u = c * (parent_n.ln() / self.node.n).sqrt();
            (1. - beta) * (self.node.q / self.node.n) + beta * (rave_q / rave_n) + u
        } else {
            let (rave_q, rave_n) = rave_table.get(&self.action).unwrap_or(&(0., 0.));
            let (rave_q, rave_n) = (*rave_q, *rave_n);
            let beta =
                (rave_n / (rave_n + self.node.n + 4. * b_squared * rave_n * self.node.n)).min(1.0);
            let u = c * parent_n.ln().sqrt(); // as if node_n = 1
            (beta) * (rave_q / rave_n) + (1. - beta) * fpu_base + u
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
