use game_sdk::bitboard::get_neighbours;
use game_sdk::{Color, GameState, PieceType};

pub fn evaluate(game_state: &GameState) -> i16 {
    (evaluate_color(game_state, Color::RED) - evaluate_color(game_state, Color::BLUE)).round()
        as i16
}
pub fn evaluate_color(game_state: &GameState, color: Color) -> f64 {
    let bee_neighbors = get_neighbours(game_state.pieces[PieceType::BEE as usize][color as usize]);
    (bee_neighbors & !game_state.occupied() & !game_state.obstacles).count_ones() as f64
}
