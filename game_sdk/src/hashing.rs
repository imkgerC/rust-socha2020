use rand::{prelude::StdRng, Rng};
use std::fmt::{Debug, Formatter, Result};

pub struct HashKeys {
    pieces: [[[u64; 128]; 2]; 5],
    beetle_stack: [[[u64; 128]; 2]; 4],
    color_to_move: u64,
}
impl HashKeys {
    pub fn gen_hash_keys() {
        let keys = HashKeys::from_seed([42; 32]);
        println!("{:?}", keys);
    }
    pub fn new() -> HashKeys {
        let mut rng = rand::thread_rng();
        HashKeys::initialize(rng)
    }
    pub fn from_seed(seed: [u8; 32]) -> HashKeys {
        let mut rng: StdRng = rand::SeedableRng::from_seed(seed);
        HashKeys::initialize(rng)
    }

    pub(crate) fn initialize<T: Rng>(mut rng: T) -> HashKeys {
        let mut pieces = [[[0u64; 128]; 2]; 5];
        pieces.iter_mut().map(|a| {
            a.iter_mut()
                .map(|b| b.iter_mut().map(|c| *c = rng.next_u64()))
        });
        let mut beetle_stack = [[[0u64; 128]; 2]; 4];
        beetle_stack.iter_mut().map(|a| {
            a.iter_mut()
                .map(|b| b.iter_mut().map(|c| *c = rng.next_u64()))
        });
        let color_to_move: u64 = rng.next_u64();
        HashKeys {
            pieces,
            beetle_stack,
            color_to_move,
        }
    }
    pub(crate) fn arr_to_string(arr: &[u64]) -> String {
        let mut res_str: String = String::new();
        res_str.push_str("[");
        for i in arr {
            res_str.push_str(&format!("{}, ", *i));
        }
        res_str.push_str("];");
        res_str
    }
}
impl Debug for HashKeys {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let mut res_str = String::new();
        let mut piece_hash_str = String::new();
        for i in self.pieces.iter() {
            piece_hash_str.push_str("[");
            for j in i.iter() {
                piece_hash_str.push_str(&HashKeys::arr_to_string(j));
            }
            piece_hash_str.push_str("], ");
        }
        let mut beetle_stack_str = String::new();
        for i in self.beetle_stack.iter() {
            beetle_stack_str.push_str("[");
            for j in i.iter() {
                beetle_stack_str.push_str(&HashKeys::arr_to_string(j));
            }
            beetle_stack_str.push_str("], ");
        }
        res_str.push_str(&format!(
            "pub const PIECE_HASH: [[[u64; 128]; 2]; 5] = {}",
            piece_hash_str
        ));
        res_str.push_str(&format!(
            "pub const BEETLE_STACK_HASH: [[[u64; 128]; 2]; 4] = {}",
            beetle_stack_str
        ));
        res_str.push_str(&format!(
            "pub const COLOR_TO_MOVE_HASH: u64 = {}",
            self.color_to_move
        ));
        write!(f, "{}", res_str)
    }
}
