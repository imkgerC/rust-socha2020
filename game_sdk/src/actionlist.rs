use crate::action::Action;
use std::fmt::{Debug, Formatter, Result};
use std::ops::{Index, IndexMut};

pub const MAX_ACTIONS: usize = 455; //TODO OBSTACLES

#[derive(Clone)]
pub struct ActionList {
    actions: [Action; MAX_ACTIONS],
    pub size: usize,
}

impl ActionList {
    pub fn find_action(&self, action: Action) -> Option<usize> {
        for i in 0..self.size {
            if self.actions[i] == action {
                return Some(i);
            }
        }
        None
    }

    pub fn swap(&mut self, a: usize, b: usize) {
        let at_a = self[a];
        self.actions[a] = self[b];
        self.actions[b] = at_a;
    }

    #[inline(always)]
    pub fn has_action(&self, index: usize) -> bool {
        index < self.size
    }

    pub fn push(&mut self, action: Action) {
        self.actions[self.size] = action;
        self.size += 1;
    }

    pub fn clear(&mut self) {
        self.size = 0;
    }
}
impl Index<usize> for ActionList {
    type Output = Action;

    #[inline(always)]
    fn index(&self, index: usize) -> &Self::Output {
        if index < self.size {
            &self.actions[index]
        } else {
            panic!(
                "Index out of bounds for ActionList, given index: {}, size: {}, actions: {:?}",
                index,
                self.size,
                self.actions[0..self.size].to_vec()
            );
        }
    }
}
impl Default for ActionList {
    fn default() -> Self {
        let actions = unsafe { std::mem::MaybeUninit::uninit().assume_init() };
        ActionList { actions, size: 0 }
    }
}
impl Debug for ActionList {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:?}", self.actions[0..self.size].to_vec())
    }
}
pub struct ActionListStack {
    pub action_lists: Vec<ActionList>,
}
impl ActionListStack {
    pub fn with_size(size: usize) -> Self {
        ActionListStack {
            action_lists: vec![ActionList::default(); size],
        }
    }
}
impl Index<usize> for ActionListStack {
    type Output = ActionList;

    fn index(&self, index: usize) -> &Self::Output {
        if index < self.action_lists.len() {
            &self.action_lists[index]
        } else {
            panic!("Can not extend ActionListStack in non mutable index");
        }
    }
}

impl IndexMut<usize> for ActionListStack {
    fn index_mut(&mut self, index: usize) -> &mut ActionList {
        if index < self.action_lists.len() {
            &mut self.action_lists[index]
        } else {
            self.action_lists
                .append(vec![ActionList::default(); index + 1 - self.action_lists.len()].as_mut());
            self.index_mut(index)
        }
    }
}
impl Default for ActionListStack {
    fn default() -> Self {
        ActionListStack::with_size(100)
    }
}
