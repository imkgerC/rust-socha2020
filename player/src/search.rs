use crate::cache::{Cache, CacheEntry, EvalCache, EvalCacheEntry, HASH_SIZE};
use crate::evaluation::evaluate;
use crate::moveordering::{MoveOrderer, STAGES};
use crate::timecontrol::Timecontrol;
use game_sdk::actionlist::ActionListStack;
use game_sdk::gamerules::{calculate_legal_moves, get_result, is_game_finished};
use game_sdk::{Action, ActionList, ClientListener, Color, GameState, MATED_IN_MAX, MATE_IN_MAX};
use std::time::Instant;

pub const STANDARD_SCORE: i16 = std::i16::MIN + 1;
pub const MAX_SEARCH_DEPTH: usize = 60;

pub struct Searcher {
    pub nodes_searched: u64,
    pub als: ActionListStack,
    pub start_time: Option<Instant>,
    pub principal_variation_table: ActionList<Action>,
    pub principal_variation_hashtable: Vec<u64>,
    pub pv_table: ActionListStack,
    pub stop_flag: bool,
    pub cache: Cache,
    pub eval_cache: EvalCache,
    pub root_plies_played: u8,
    pub tc: Timecontrol,
    pub killer_moves: [[Option<Action>; 2]; MAX_SEARCH_DEPTH],
    pub hh_score: [[[usize; 122]; 122]; 2],
    pub bf_score: [[[usize; 122]; 122]; 2],
    pub cutoff_stats: Vec<u64>,
}

impl Searcher {
    pub fn new() -> Self {
        Searcher {
            nodes_searched: 0,
            als: ActionListStack::with_size(MAX_SEARCH_DEPTH),
            start_time: None,
            principal_variation_table: ActionList::default(),
            principal_variation_hashtable: Vec::with_capacity(MAX_SEARCH_DEPTH),
            pv_table: ActionListStack::with_size(MAX_SEARCH_DEPTH),
            stop_flag: false,
            cache: Cache::with_size(HASH_SIZE),
            eval_cache: EvalCache::with_size(HASH_SIZE),
            root_plies_played: 0,
            tc: Timecontrol::MoveTime(1800),
            killer_moves: [[None; 2]; MAX_SEARCH_DEPTH],
            hh_score: [[[0usize; 122]; 122]; 2],
            bf_score: [[[1usize; 122]; 122]; 2],
            cutoff_stats: vec![0; 60],
        }
    }
    pub fn with_tc(tc: Timecontrol) -> Self {
        let mut res = Searcher::new();
        res.tc = tc;
        res
    }

    pub fn search_move(&mut self, game_state: &GameState) -> Action {
        println!("Searching state w/ fen:{}", game_state.to_fen());
        let mut al = ActionList::default();
        calculate_legal_moves(&game_state, &mut al);
        if al.size == 0 {
            panic!("There are no legal moves in this position! What should I return?");
        }
        let mut game_state = game_state.clone();
        self.nodes_searched = 0;
        self.start_time = Some(Instant::now());
        self.principal_variation_table.clear();
        self.principal_variation_hashtable.clear();
        self.stop_flag = false;
        self.root_plies_played = game_state.ply;
        self.killer_moves = [[None; 2]; MAX_SEARCH_DEPTH];
        self.cutoff_stats = vec![0; 60];
        for i in 0..2 {
            for j in 0..122 {
                for k in 0..122 {
                    self.hh_score[i][j][k] /= 8;
                    self.bf_score[i][j][k] = (self.bf_score[i][j][k] / 8).max(1);
                }
            }
        }

        let mut score = STANDARD_SCORE;
        let current_max_depth = (61 - game_state.ply) as usize;
        for depth in 1..=current_max_depth {
            let new_score = principal_variation_search(
                self,
                &mut game_state,
                0,
                depth,
                STANDARD_SCORE,
                -STANDARD_SCORE,
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
                "info depth {} score {} bestmove {:?} nodes {} nps {:.2} time {} hashfull {} pv {:?}",
                depth,
                score,
                self.principal_variation_table[0],
                self.nodes_searched,
                nps,
                self.start_time.unwrap().elapsed().as_millis(),
                self.cache.fill_status(),
                self.principal_variation_table
            );
            /*println!("{:?}", self.cutoff_stats);
            let sum: u64 = self.cutoff_stats.iter().sum();
            println!(
                "{:?}",
                self.cutoff_stats
                    .iter()
                    .map(|s| *s as f64 / (sum as f64).max(1.))
                    .collect::<Vec<f64>>()
            );*/
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
        self.search_move(state)
    }
}
pub fn principal_variation_search(
    searcher: &mut Searcher,
    game_state: &mut GameState,
    current_depth: usize,
    depth_left: usize,
    mut alpha: i16,
    mut beta: i16,
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
    let original_alpha = alpha;

    if searcher.nodes_searched % 4096 == 0 {
        if searcher.tc.time_over(
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
    if searcher.nodes_searched % 10000000 == 0 {
        println!(
            "info nps {}",
            searcher.nodes_searched as f64 / (searcher.start_time.unwrap().elapsed().as_secs_f64())
        );
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

    //Mate distance pruning
    {
        //I know I am not getting mated in this position
        alpha = alpha.max((MATE_IN_MAX + 60 - current_depth as i16 - 1) * -1);
        //And I am at best mating in the next position
        beta = beta.min(MATE_IN_MAX + 60 - current_depth as i16 - 1);
        if alpha >= beta {
            return alpha;
        }
    }

    //TODO: Quiescence search
    if depth_left <= 0 {
        let ce = searcher.eval_cache.lookup(game_state.hash);
        if let Some(ce) = ce {
            return ce.score;
        }
        //return eval
        let evaluation = evaluate(game_state) * color;
        searcher.eval_cache.insert(
            game_state.hash,
            EvalCacheEntry {
                upper_hash: (game_state.hash >> 32) as u32,
                lower_hash: (game_state.hash & 0xFFFFFFFF) as u32,
                score: evaluation,
            },
        );
        return evaluation;
    }

    let pv_action = if searcher.principal_variation_table.size > current_depth
        && searcher.principal_variation_hashtable[current_depth] == game_state.hash
    {
        Some(searcher.principal_variation_table[current_depth])
    } else {
        None
    };

    //TT-Lookup
    let mut tt_action: Option<Action> = None;
    {
        let ce = searcher.cache.lookup(game_state.hash);
        if let Some(ce) = ce {
            if ce.depth >= depth_left as u8
                && ((game_state.ply + depth_left as u8) < 60 && ce.plies + ce.depth < 60
                    || (game_state.ply + depth_left as u8) >= 60 && ce.plies + ce.depth >= 60)
            {
                let tt_score = if ce.score >= MATE_IN_MAX {
                    ce.score - current_depth as i16
                } else if ce.score <= MATED_IN_MAX {
                    ce.score + current_depth as i16
                } else {
                    ce.score
                };
                let mate_length = if ce.score.abs() >= MATE_IN_MAX {
                    MATE_IN_MAX + 60 - ce.score.abs()
                } else {
                    0
                };
                let draw_length = if tt_score == 0 && ce.plies + ce.depth >= 60 {
                    Some(60 - ce.plies)
                } else {
                    None
                };

                if (game_state.ply + mate_length as u8) <= 60
                    && !root
                    && (draw_length.is_none() || game_state.ply + draw_length.unwrap() == 60)
                    && (!ce.alpha && !ce.beta
                        || ce.beta && tt_score >= beta
                        || ce.alpha && alpha >= tt_score)
                {
                    return tt_score;
                }
            }
            tt_action = Some(ce.action);
        }
    }
    //TODO: Pruning
    let mut wouldnmp = false;
    //Null move Pruning
    if !pv_node && (!game_state.must_player_place_bee() || game_state.has_player_placed_bee() )// not necessary but should be speedup
        && depth_left > 3
        && (game_state.ply + depth_left as u8) < 60
        && (game_state
        .valid_set_destinations(game_state.color_to_move)
        .count_ones()
        > 0)
        && evaluate(&game_state) * color >= beta
    {
        let action = Action::SkipMove;
        game_state.make_action(action);
        let following_score = -principal_variation_search(
            searcher,
            game_state,
            current_depth + 1,
            (depth_left - 3).max(1) as usize,
            -beta,
            -beta + 1,
        );
        game_state.unmake_action(action);
        if following_score >= beta {
            return following_score;
            //wouldnmp = true;
        }
    }

    let mut current_max_score = STANDARD_SCORE;

    let mut move_orderer = MoveOrderer::with_stages(&STAGES);
    let mut i = 0;
    while let Some(action) =
        move_orderer.next(game_state, searcher, current_depth, pv_action, tt_action)
    {
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
            );
            if following_score > alpha {
                following_score = -principal_variation_search(
                    searcher,
                    game_state,
                    current_depth + 1,
                    depth_left - 1,
                    -beta,
                    -alpha,
                );
            }
            following_score
        };
        game_state.unmake_action(action);
        if following_score > current_max_score && !searcher.stop_flag {
            current_max_score = following_score;
            searcher.pv_table[current_depth].clear();
            searcher.pv_table[current_depth].push(action);
            //Set Pv
            if pv_node {
                for i in 0..searcher.pv_table[current_depth + 1].size {
                    let action = searcher.pv_table[current_depth + 1][i];
                    searcher.pv_table[current_depth].push(action);
                }
            }
        }
        alpha = alpha.max(following_score);

        let (from, to) = match action {
            Action::SkipMove => (121, 121),
            Action::DragMove(_, from, to) => (from, to),
            Action::SetMove(_, to) => (121, to),
        };
        if alpha >= beta {
            if Some(action) != searcher.killer_moves[current_depth][0]
                && Some(action) != searcher.killer_moves[current_depth][1]
            {
                let before = searcher.killer_moves[current_depth][0];
                searcher.killer_moves[current_depth][0] = Some(action);
                searcher.killer_moves[current_depth][1] = before;
                searcher.hh_score[game_state.color_to_move as usize][from as usize][to as usize] +=
                    depth_left;
            }
            break;
        } else {
            searcher.bf_score[game_state.color_to_move as usize][from as usize][to as usize] +=
                depth_left;
        }
        i += 1;
    }
    if wouldnmp && alpha >= beta {
        searcher.cutoff_stats[0] += 1;
    } else if wouldnmp && alpha < beta {
        searcher.cutoff_stats[1] += 1;
        //println!("{}",game_state);
    }
    if !searcher.stop_flag && i == 0 && current_max_score == STANDARD_SCORE {
        panic!("No legal move found and tried in a position! This should never occur!");
    }
    //Make TT entry
    if !searcher.stop_flag {
        let score = if current_max_score.abs() >= MATE_IN_MAX {
            let mate_length = MATE_IN_MAX + 60 - current_max_score.abs();
            assert!((current_depth as i16) < mate_length);
            let mate_length_from_now_on = mate_length - current_depth as i16;
            (MATE_IN_MAX + 60 - mate_length_from_now_on as i16)
                * if current_max_score >= MATE_IN_MAX {
                    1
                } else {
                    -1
                }
        } else {
            current_max_score
        };
        searcher.cache.insert(
            game_state.hash,
            CacheEntry {
                upper_hash: (game_state.hash >> 32) as u32,
                lower_hash: (game_state.hash & 0xFFFFFFFF) as u32,
                action: searcher.pv_table[current_depth][0],
                score,
                alpha: current_max_score <= original_alpha,
                beta: alpha >= beta,
                depth: depth_left as u8,
                plies: game_state.ply,
            },
            searcher.root_plies_played,
        );
    }
    current_max_score
}
