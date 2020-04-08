use crate::evaluation::evaluate;
use crate::timecontrol::Timecontrol;
use game_sdk::actionlist::ActionListStack;
use game_sdk::gamerules::{calculate_legal_moves, get_result, is_game_finished};
use game_sdk::{Action, ActionList, ClientListener, Color, GameState};
use std::time::Instant;

pub const MATE_IN_MAX: i16 = 30000;
pub const MATED_IN_MAX: i16 = -MATE_IN_MAX;
pub const STANDARD_SCORE: i16 = std::i16::MIN + 1;

pub struct Searcher {
    pub nodes_searched: u64,
    pub als: ActionListStack,
    pub start_time: Option<Instant>,
    pub principal_variation_table: ActionList,
    pub principal_variation_hashtable: Vec<u64>,
    pub pv_table: ActionListStack,
    pub stop_flag: bool,
}

impl Searcher {
    pub fn new() -> Self {
        Searcher {
            nodes_searched: 0,
            als: ActionListStack::with_size(60),
            start_time: None,
            principal_variation_table: ActionList::default(),
            principal_variation_hashtable: Vec::with_capacity(60),
            pv_table: ActionListStack::with_size(60),
            stop_flag: false,
        }
    }

    pub fn search_move(&mut self, game_state: &GameState, tc: Timecontrol) -> Action {
        println!("Searching state w/ fen:{}", game_state.to_fen());
        let mut game_state = game_state.clone();
        self.nodes_searched = 0;
        self.start_time = Some(Instant::now());
        self.principal_variation_table.clear();
        self.principal_variation_hashtable.clear();
        self.stop_flag = false;
        let mut score = STANDARD_SCORE;
        for depth in 1..61 {
            let new_score = principal_variation_search(
                self,
                &mut game_state,
                0,
                depth,
                STANDARD_SCORE,
                -STANDARD_SCORE,
                tc,
            );
            if self.stop_flag {
                break;
            }
            score = new_score;
            self.principal_variation_table = self.pv_table[0].clone();
            let mut toy_state = game_state.clone();
            self.principal_variation_hashtable.clear();
            for i in 0..self.principal_variation_table.size {
                self.principal_variation_hashtable.push(toy_state.hash);
                toy_state.make_action(self.principal_variation_table[i]);
            }
            let nps =
                self.nodes_searched as f64 / (self.start_time.unwrap().elapsed().as_secs_f64());
            println!(
                "info depth {} score {} bestmove {:?} nodes {} nps {:.2} pv {:?}",
                depth,
                score,
                self.principal_variation_table[0],
                self.nodes_searched,
                nps,
                self.principal_variation_table
            );
        }
        println!(
            "Finished search with move {:?} and score {}, pv: {:?}",
            self.principal_variation_table[0], score, self.principal_variation_table
        );
        //let score = principal_variation_search(&mut self, game_state, )
        self.principal_variation_table[0]
    }
}
impl ClientListener for Searcher {
    fn on_move_request(&mut self, state: &GameState) -> Action {
        self.search_move(state, Timecontrol::MoveTime(1800))
    }
}
pub fn principal_variation_search(
    searcher: &mut Searcher,
    game_state: &mut GameState,
    current_depth: usize,
    depth_left: isize,
    mut alpha: i16,
    beta: i16,
    tc: Timecontrol,
) -> i16 {
    searcher.nodes_searched += 1;
    //clear_pv
    searcher.pv_table[current_depth].clear();
    let root = current_depth == 0;
    let pv_node = beta > 1 + alpha;
    let color = if game_state.color_to_move == Color::RED {
        1
    } else {
        -1
    };

    if searcher.nodes_searched % 4096 == 0 {
        if tc.time_over(
            searcher
                .start_time
                .expect("No start time set")
                .elapsed()
                .as_millis() as u64,
        ) {
            searcher.stop_flag = true;
            return STANDARD_SCORE;
        }
    }
    //Check game over
    if is_game_finished(game_state) {
        let winner = get_result(game_state);
        if winner.is_none() {
            return 0;
        } else if winner.unwrap() == Color::RED {
            return (MATE_IN_MAX + 60 - current_depth as i16) * color;
        } else {
            return (MATE_IN_MAX + 60 - current_depth as i16) * -color;
        }
    }
    debug_assert!(current_depth < 60);

    //TODO: Mate distance pruning

    //TODO: Quiescence search
    if depth_left <= 0 {
        //return eval
        let evaluation = evaluate(game_state) * color;
        return evaluation;
    }

    let pv_action = if searcher.principal_variation_table.size > current_depth
        && searcher.principal_variation_hashtable[current_depth] == game_state.hash
    {
        Some(searcher.principal_variation_table[current_depth])
    } else {
        None
    };

    //TODO: TT Lookup
    let tt_action: Option<Action> = None;

    //TODO: Pruning

    let mut current_max_score = STANDARD_SCORE;
    calculate_legal_moves(game_state, &mut searcher.als[current_depth]);
    if pv_action.is_some() {
        let index = searcher.als[current_depth]
            .find_action(pv_action.unwrap())
            .expect("Pv move not found in movelist");
        searcher.als[current_depth].swap(0, index);
    }
    //TODO move sorting
    for i in 0..searcher.als[current_depth].size {
        let action = searcher.als[current_depth][i];
        game_state.make_action(action);
        //TODO: Forward pruning & late-move-reductions
        let following_score = if depth_left <= 2 || !pv_node || i == 0 {
            //Full window
            -principal_variation_search(
                searcher,
                game_state,
                current_depth + 1,
                depth_left - 1,
                -beta,
                -alpha,
                tc,
            )
        } else {
            //Null window
            let mut following_score = -principal_variation_search(
                searcher,
                game_state,
                current_depth + 1,
                depth_left - 1,
                -alpha - 1,
                -alpha,
                tc,
            );
            if following_score > alpha {
                following_score = -principal_variation_search(
                    searcher,
                    game_state,
                    current_depth + 1,
                    depth_left - 1,
                    -beta,
                    -alpha,
                    tc,
                );
            }
            following_score
        };
        game_state.unmake_action(action);
        if following_score > current_max_score && !searcher.stop_flag {
            current_max_score = following_score;
            //Set Pv
            if pv_node {
                searcher.pv_table[current_depth].clear();
                searcher.pv_table[current_depth].push(action);
                for i in 0..searcher.pv_table[current_depth + 1].size {
                    let action = searcher.pv_table[current_depth + 1][i];
                    searcher.pv_table[current_depth].push(action);
                }
            }
        }
        alpha = alpha.max(following_score);

        //TODO Update history scores & killers
        if alpha >= beta {
            break;
        }
    }
    //TODO Make TT Entry
    current_max_score
}
