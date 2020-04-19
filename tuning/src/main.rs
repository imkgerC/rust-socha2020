use game_sdk::misc::FenReader;
use game_sdk::{Color, GameState};
use player::evaluation::{evaluate, evaluate_with_parameters, EvaluationParameters};

pub const K: f64 = 1.5;
pub struct Dataset(Vec<LabelledGameState>);
pub fn sigmoid(k: f64, x: f64) -> f64 {
    1. / (1. + 10f64.powf(-k * x / 400.0))
}
impl Dataset {
    pub fn get_mean_evaluation_error(&self, k: f64) -> f64 {
        let mut res = 0.;
        for lgs in self.0.iter() {
            res += (lgs.1 - sigmoid(k, evaluate(&lgs.0) as f64)).powf(2.);
        }
        res /= self.0.len() as f64;
        res
    }

    pub fn get_mean_evaluation_error_with_params(&self, k: f64, params: [f64; 10]) -> f64 {
        let params = EvaluationParameters::from_array(params);
        let mut res = 0.;
        for lgs in self.0.iter() {
            res += (lgs.1 - sigmoid(k, evaluate_with_parameters(&lgs.0, &params) as f64)).powf(2.);
        }
        res /= self.0.len() as f64;
        res
    }
}
pub struct LabelledGameState(GameState, f64);
impl LabelledGameState {
    pub fn from_path(path: &str) -> Dataset {
        let mut res = Vec::with_capacity(10000);
        let (mut games, mut redwins, mut draws, mut bluewins) = (0, 0, 0, 0);
        for game in FenReader::from_path(path).into_game_reader() {
            let game_res = game.1.clone();
            game.0.into_iter().for_each(|state| {
                let val = if game_res == Some(Color::RED) {
                    1.0
                } else if game_res == Some(Color::BLUE) {
                    0.0
                } else {
                    0.5
                };
                res.push(LabelledGameState(state, val))
            });
            if game_res == Some(Color::RED) {
                redwins += 1;
            } else if game_res == Some(Color::BLUE) {
                bluewins += 1;
            } else {
                draws += 1;
            }
            games += 1;
        }
        println!(
            "Parsed {} games with {} states!\nRedWins: {}, Draws: {}, BlueWins: {}",
            games,
            res.len(),
            redwins,
            draws,
            bluewins
        );
        Dataset(res)
    }
}

#[allow(dead_code)]
fn find_k(labelled_states: &Dataset) {
    let mut best_k = 0.1;
    let mut best_score = 1.0;
    let mut k = 0.1;
    while k <= 10. {
        let eval_error = labelled_states.get_mean_evaluation_error(k);
        if eval_error < best_score {
            best_score = eval_error;
            best_k = k;
        }
        k += 0.01;
    }
    println!("Best K :{}, Error: {}", best_k, best_score);
}

fn local_parameter_search(labelled_states: &Dataset) {
    let mut best_params = player::evaluation::DEFAULT_ARRAY;
    let mut best_error = labelled_states.get_mean_evaluation_error_with_params(K, best_params);
    let mut improved = true;
    let mut iteration = 0;
    while improved {
        iteration += 1;
        improved = false;
        for i in 0..best_params.len() {
            let mut new_params = best_params;
            new_params[i] = best_params[i] + (best_params[i].abs() * 0.1).max(0.05);
            let new_error = labelled_states.get_mean_evaluation_error_with_params(K, new_params);
            if new_error < best_error {
                improved = true;
                best_error = new_error;
                best_params = new_params;
            } else {
                new_params[i] = best_params[i] - (best_params[i].abs() * 0.1).max(0.05);
                let new_error =
                    labelled_states.get_mean_evaluation_error_with_params(K, new_params);
                if new_error < best_error {
                    improved = true;
                    best_error = new_error;
                    best_params = new_params;
                }
            }
        }
        println!(
            "finished iteration {} with error {}, params {:?}",
            iteration, best_error, best_params
        );
    }
}

fn main() {
    let labelled_states = LabelledGameState::from_path("./referee_logs/fens.txt");
    println!(
        "Average evaluation error: {}",
        labelled_states.get_mean_evaluation_error(K)
    );
    local_parameter_search(&labelled_states);
}
