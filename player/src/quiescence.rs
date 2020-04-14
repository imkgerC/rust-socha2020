use crate::cache::{CacheEntry, EvalCacheEntry};
use crate::evaluation::evaluate;
use crate::moveordering::{MoveOrderer, QSTAGES};
use crate::search::Searcher;
use game_sdk::gamerules::{get_result, is_game_finished};
use game_sdk::{Action, Color, GameState, MATED_IN_MAX, MATE_IN_MAX, STANDARD_SCORE};

pub fn qsearch(
    searcher: &mut Searcher,
    game_state: &mut GameState,
    current_depth: usize,
    depth_left: isize,
    mut alpha: i16,
    beta: i16,
) -> i16 {
    searcher.nodes_searched += 1;
    let color = if game_state.color_to_move == Color::RED {
        1
    } else {
        -1
    };
    let original_alpha = alpha;
    searcher.pv_table[current_depth].clear();

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

    let standing_pat = {
        let ce = searcher.eval_cache.lookup(game_state.hash);
        if let Some(ce) = ce {
            ce.score
        } else {
            let evaluation = evaluate(game_state) * color;
            searcher.eval_cache.insert(
                game_state.hash,
                EvalCacheEntry {
                    upper_hash: (game_state.hash >> 32) as u32,
                    lower_hash: (game_state.hash & 0xFFFFFFFF) as u32,
                    score: evaluation,
                },
            );
            evaluation
        }
    };

    // pruning
    if alpha > standing_pat + 50 {
        return standing_pat;
    }
    alpha = alpha.max(standing_pat);
    if alpha >= beta {
        return alpha;
    }

    let pv_action = if searcher.principal_variation_table.size > current_depth
        && searcher.principal_variation_hashtable[current_depth] == game_state.hash
    {
        Some(searcher.principal_variation_table[current_depth])
    } else {
        None
    };

    let mut tt_action: Option<Action> = None;
    {
        let ce = searcher.cache.lookup(game_state.hash);
        if let Some(ce) = ce {
            if (game_state.ply + 1) < 60 && ce.plies + ce.depth < 60
                || (game_state.ply + 1) >= 60 && ce.plies + ce.depth >= 60
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

    let current_max_score = standing_pat;

    let mut move_orderer = MoveOrderer::with_stages(&QSTAGES);
    while let Some(action) =
        move_orderer.next(game_state, searcher, current_depth, pv_action, tt_action)
    {
        game_state.make_action(action);
        let following_score = -qsearch(
            searcher,
            game_state,
            current_depth + 1,
            depth_left - 1,
            -beta,
            -alpha,
        );
        game_state.unmake_action(action);
        if searcher.stop_flag {
            break;
        }
        if following_score > current_max_score {
            searcher.pv_table[current_depth].clear();
            searcher.pv_table[current_depth].push(action);
            // do not add following pv as is trash anyways
        }
        current_max_score.max(following_score);
        alpha.max(following_score);
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
                    1 as usize;
            }
            break;
        } else {
            searcher.bf_score[game_state.color_to_move as usize][from as usize][to as usize] +=
                1 as usize;
        }
    }

    // Make TT entry
    if !searcher.stop_flag && searcher.pv_table[current_depth].size > 0 {
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
                depth: 0u8,
                plies: game_state.ply,
            },
            searcher.root_plies_played,
        );
    }

    current_max_score
}
