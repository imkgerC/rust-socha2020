use crate::action::Action;

pub const MAX_ACTIONS: usize = 440;
pub struct ActionList {
    actions: [Action; MAX_ACTIONS],
}

impl ActionList {}
