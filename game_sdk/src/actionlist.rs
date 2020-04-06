use crate::action::Action;
use std::ops::{Index, IndexMut};

pub const MAX_ACTIONS: usize = 440;

#[derive(Clone)]
pub struct ActionList {
    actions: [Action; MAX_ACTIONS],
    size: usize,
}

impl ActionList {
    #[inline(always)]
    pub fn get_action(&self, index: usize) -> &Action {
        //TODO Decide: Action Copy or not? Move or not?
        if index < self.size {
            self.actions
                .get(index)
                .expect("ActionList size exceeded MAX_ACTIONS.")
        } else {
            panic!(&format!(
                "Index out of bounds for ActionList, given index: {}, size: {}, actions: {:?}",
                index, self.size, self.actions
            ))
        }
    }
    pub fn iter(&self) -> ActionListIter {
        ActionListIter {
            action_list: self,
            curr_index: 0,
        }
    }

    #[inline(always)]
    pub fn has_action(&self, index: usize) -> bool {
        index < self.size
    }
}

struct ActionListIter<'a> {
    action_list: &'a ActionList,
    curr_index: usize,
}
impl Iterator for ActionListIter {
    type Item = Action;

    fn next(&mut self) -> Option<Self::Item> {
        if self.action_list.has_action(self.curr_index) {
            let res = self.action_list.get_action(self.curr_index);
            self.curr_index += 1;
            Some(*res)
        } else {
            None
        }
    }
}

impl Default for ActionList {
    fn default() -> Self {
        let actions = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
        ActionList { actions, size: 0 }
    }
}

pub struct ActionListStack {
    action_lists: Vec<ActionList>,
}
impl ActionListStack {
    pub fn with_size(size: usize) -> Self {
        ActionListStack {
            action_lists: vec![ActionList::default(); size],
        }
    }
}
impl IndexMut<usize> for ActionListStack {
    fn index_mut(&mut self, index: usize) -> &mut ActionList {
        if index < self.action_lists.len() {
            &mut self.action_lists[index]
        } else {
            unimplemented!("");
        }
    }
}
impl Default for ActionListStack {
    fn default() -> Self {
        ActionListStack::with_size(100)
    }
}
