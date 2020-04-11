use crate::moveordering::MoveOrderingStage::{
    BadPins, GenerateMoves, Killer, PVMove, PinInitialization, Pins, Quiet, QuietInitialization,
    TTMove,
};
use crate::search::Searcher;
use game_sdk::gamerules::calculate_legal_moves;
use game_sdk::{Action, ActionList, Color, GameState, PieceType};

pub const ATTACKER_VALUE: [f64; 5] = [5., 1., 4., 3., 2.];
pub const TARGET_VALUE: [f64; 5] = [500., 400., 300., 100., 200.];
pub const STAGES: [MoveOrderingStage; 9] = [
    GenerateMoves,
    PVMove,
    TTMove,
    PinInitialization,
    Pins,
    Killer,
    QuietInitialization,
    Quiet,
    BadPins,
];
pub enum MoveOrderingStage {
    GenerateMoves,
    PVMove,
    TTMove,
    PinInitialization,
    Pins,
    Killer,
    QuietInitialization,
    Quiet,
    BadPins,
}
pub struct MoveOrderer {
    pub stage: usize,
    pub stages: &'static [MoveOrderingStage],
    score_list: ActionList<Option<f64>>,
}
impl MoveOrderer {
    pub fn with_stages(stages: &'static [MoveOrderingStage]) -> MoveOrderer {
        MoveOrderer {
            stage: 0,
            stages,
            score_list: ActionList::default(),
        }
    }
    #[inline(always)]
    fn remove_action_at(&mut self, at: usize, al: &mut ActionList<Action>) {
        al.remove_index(at);
        self.score_list.remove_index(at);
    }

    #[inline(always)]
    fn remove_action(&mut self, action: Action, al: &mut ActionList<Action>) {
        let index = al
            .find_action(action)
            .expect("Can not remove action which is not in action list");
        self.remove_action_at(index, al);
    }

    #[inline(always)]
    fn highest_score(
        &mut self,
        al: &mut ActionList<Action>,
        remove_tresh: Option<f64>,
    ) -> Option<(Action, f64)> {
        let mut highest_score = None;
        let mut highest_index = None;
        for i in 0..al.size {
            if self.score_list[i].is_some()
                && (highest_score.is_none() || self.score_list[i].unwrap() > highest_score.unwrap())
            {
                highest_score = self.score_list[i];
                highest_index = Some(i);
            }
        }
        if highest_score.is_some() {
            let action = al[highest_index.unwrap()];
            if remove_tresh.is_none()
                || remove_tresh.is_some() && highest_score.unwrap() >= remove_tresh.unwrap()
            {
                self.remove_action(al[highest_index.unwrap()], al);
            }
            Some((action, highest_score.unwrap()))
        } else {
            None
        }
    }

    #[inline(always)]
    fn pin_score(
        pinner_type: PieceType,
        pinned_type: PieceType,
        color: Color,
        color_to_move: Color,
    ) -> f64 {
        let res = TARGET_VALUE[pinned_type as usize] - ATTACKER_VALUE[pinner_type as usize];
        res * if color == color_to_move { -1.0 } else { 0.0 }
    }

    pub fn next(
        &mut self,
        game_state: &GameState,
        searcher: &mut Searcher,
        current_depth: usize,
        pv_action: Option<Action>,
        tt_action: Option<Action>,
    ) -> Option<Action> {
        if self.stage >= self.stages.len() {
            return None;
        }
        match self.stages[self.stage] {
            GenerateMoves => {
                self.stage += 1;
                calculate_legal_moves(&game_state, &mut searcher.als[current_depth]);
                for _ in 0..searcher.als[current_depth].size {
                    self.score_list.push(None);
                }
                self.next(game_state, searcher, current_depth, pv_action, tt_action)
            }
            PVMove => {
                self.stage += 1;
                if pv_action.is_some() {
                    self.remove_action(pv_action.unwrap(), &mut searcher.als[current_depth]);
                    Some(pv_action.unwrap())
                } else {
                    self.next(game_state, searcher, current_depth, pv_action, tt_action)
                }
            }
            TTMove => {
                self.stage += 1;
                if tt_action.is_some() && tt_action != pv_action {
                    self.remove_action(tt_action.unwrap(), &mut searcher.als[current_depth]);
                    Some(tt_action.unwrap())
                } else {
                    self.next(game_state, searcher, current_depth, pv_action, tt_action)
                }
            }
            PinInitialization => {
                let al = &mut searcher.als[current_depth];
                for i in 0..al.size {
                    if let Some(info) = game_state.get_pin_info(al[i]) {
                        self.score_list.overwrite(
                            i,
                            Some(MoveOrderer::pin_score(
                                info.0,
                                info.2,
                                info.1,
                                game_state.color_to_move,
                            )),
                        );
                    }
                }
                self.stage += 1;
                self.next(game_state, searcher, current_depth, pv_action, tt_action)
            }
            Pins => {
                if let Some((res, score)) =
                    self.highest_score(&mut searcher.als[current_depth], Some(0.))
                {
                    if score >= 0. {
                        Some(res)
                    } else {
                        self.stage += 1;
                        self.next(game_state, searcher, current_depth, pv_action, tt_action)
                    }
                } else {
                    self.stage += 1;
                    self.next(game_state, searcher, current_depth, pv_action, tt_action)
                }
            }
            Killer => {
                let mut found_index = None;
                for i in 0..searcher.als[current_depth].size {
                    if Some(searcher.als[current_depth][i])
                        == searcher.killer_moves[current_depth][0]
                        || Some(searcher.als[current_depth][i])
                            == searcher.killer_moves[current_depth][1]
                    {
                        found_index = Some(i);
                        break;
                    }
                }
                if found_index.is_some() {
                    let action = searcher.als[current_depth][found_index.unwrap()];
                    self.remove_action_at(found_index.unwrap(), &mut searcher.als[current_depth]);
                    Some(action)
                } else {
                    self.stage += 1;
                    self.next(game_state, searcher, current_depth, pv_action, tt_action)
                }
            }
            QuietInitialization => {
                for i in 0..searcher.als[current_depth].size {
                    if self.score_list[i].is_none() {
                        let (from, to) = match searcher.als[current_depth][i] {
                            Action::SkipMove => (121, 121),
                            Action::DragMove(_, from, to) => (from, to),
                            Action::SetMove(_, to) => (121, to),
                        };
                        self.score_list.overwrite(
                            i,
                            Some(
                                searcher.hh_score[game_state.color_to_move as usize][from as usize]
                                    [to as usize] as f64
                                    / searcher.bf_score[game_state.color_to_move as usize]
                                        [from as usize][to as usize]
                                        as f64,
                            ),
                        )
                    }
                }
                self.stage += 1;
                self.next(game_state, searcher, current_depth, pv_action, tt_action)
            }
            Quiet => {
                if let Some((res, score)) =
                    self.highest_score(&mut searcher.als[current_depth], Some(0.))
                {
                    if score >= 0. {
                        Some(res)
                    } else {
                        self.stage += 1;
                        self.next(game_state, searcher, current_depth, pv_action, tt_action)
                    }
                } else {
                    self.stage += 1;
                    self.next(game_state, searcher, current_depth, pv_action, tt_action)
                }
            }
            BadPins => {
                if let Some((res, _)) = self.highest_score(&mut searcher.als[current_depth], None) {
                    Some(res)
                } else {
                    self.stage += 1;
                    self.next(game_state, searcher, current_depth, pv_action, tt_action)
                }
            }
        }
    }
}
