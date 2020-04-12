use crate::cache::EvalCacheEntry;
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
    // No pv in qsearch

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

    alpha = alpha.max(standing_pat);
    if alpha >= beta {
        return alpha;
    }

    let mut tt_action: Option<Action> = None;
    {
        let ce = searcher.cache.lookup(game_state.hash);
        if let Some(ce) = ce {
            if ce.depth > 0 // always is, as q is not added to tt yet
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
    while let Some(action) = move_orderer.next(game_state, searcher, current_depth, None, tt_action)
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
        if searcher.stop_flag {
            break;
        }
        current_max_score.max(following_score);
        alpha.max(following_score);
        if alpha >= beta {
            break;
        }
        game_state.unmake_action(action);
    }
    // TODO: make tt entry?

    current_max_score
}
