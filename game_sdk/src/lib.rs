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
pub use actionlist::ActionList;
pub use gamestate::Color;
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
