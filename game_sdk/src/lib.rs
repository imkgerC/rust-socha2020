mod action;
mod actionlist;
mod bitboard;
pub mod gamerules;
mod gamestate;
mod piece_type;

pub use gamestate::GameState;
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
