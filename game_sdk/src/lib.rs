extern crate rand;
mod action;
pub mod actionlist;
pub mod bitboard;
mod fieldtype;
pub mod gamerules;
mod gamestate;
pub(crate) mod hashing;
pub mod misc;
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
    use crate::misc::FenReader;

    #[test]
    fn perftsuite() {
        for (state, perfts) in FenReader::from_path("../perft_values").into_iter() {
            let perfts: Vec<u64> = perfts
                .split(" ")
                .into_iter()
                .map(|s| s.replace("\r", "").parse::<u64>().unwrap())
                .collect();
            for perft in perfts.iter().enumerate() {
                assert_eq!(state.perft(perft.0 + 1), *perft.1)
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
