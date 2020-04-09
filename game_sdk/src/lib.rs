extern crate rand;
mod action;
pub mod actionlist;
pub mod bitboard;
mod fieldtype;
pub mod gamerules;
mod gamestate;
pub(crate) mod hashing;
pub mod neighbor_magic;
mod piece_type;
pub use action::Action;
pub use actionlist::ActionList;
pub use gamestate::Color;
pub use gamestate::GameState;
pub use hashing::HashKeys;
pub use neighbor_magic::get_accessible_neighbors;
pub use piece_type::PieceType;

#[cfg(test)]
mod tests {
    use crate::GameState;

    #[test]
    fn perftsuite() {
        let perft_contents = std::fs::read_to_string("../perft_values").unwrap();
        let perft_contents = perft_contents.replace("\r", "");
        let lines: Vec<&str> = perft_contents.split("\n").collect();
        for line in lines {
            let s: Vec<&str> = line.split("/").collect();
            let fen = s[0];
            let perfts: Vec<&str> = s[1].split(" ").collect();
            let state = GameState::from_fen(fen.to_owned());
            for perft in perfts.iter().enumerate() {
                let depth = perft.0 + 1;
                let value = perft.1.parse::<u64>().unwrap();
                assert_eq!(state.perft(depth), value)
            }
        }
    }
}

/// Trait that needs to be implemented for every Player
/// The "on"-methods are called on the events
pub trait ClientListener {
    /// This function is called whenever a memento message is received from the
    /// server. It is given a gamestate-struct
    fn on_update_state(&mut self, _state: &GameState) {}

    /// On connection with the server gets our PlayerColor inside the WelcomeMessage
    fn on_welcome_message(&mut self, _color: Color) {}

    /// On every Request a Move is requested. Needs to be implemented by every client.
    /// Implements most of the game-playing logic inside this method in a typical client
    fn on_move_request(&mut self, state: &GameState) -> Action;
}
