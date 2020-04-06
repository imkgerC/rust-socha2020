mod gamestate;
pub mod gamerules;
mod bitboard;
mod action;
mod piece_type;
mod actionlist;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
