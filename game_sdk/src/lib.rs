extern crate rand;
mod action;
mod actionlist;
mod bitboard;
mod fieldtype;
pub mod gamerules;
mod gamestate;
mod hashing;
mod piece_type;

pub use action::Action;
pub use gamestate::GameState;
pub use hashing::HashKeys;
pub use piece_type::PieceType;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
