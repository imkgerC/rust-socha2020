use game_sdk::bitboard::get_neighbours;
use game_sdk::{get_accessible_neighbors, Color, GameState, PieceType};

pub fn evaluate(game_state: &GameState) -> i16 {
    (evaluate_color(game_state, Color::RED) - evaluate_color(game_state, Color::BLUE)).round()
        as i16
}
pub fn evaluate_color(game_state: &GameState, color: Color) -> f64 {
    let occupied = game_state.occupied();
    let obstacles = game_state.obstacles;

    let bee = game_state.pieces[PieceType::BEE as usize][color as usize];
    let bee_neighbors = get_neighbours(bee);
    let bee_moves = get_accessible_neighbors(occupied, obstacles, bee).count_ones();
    let free_bee_fields = (bee_neighbors & !occupied & !obstacles).count_ones();
    let our_set_fields = (get_neighbours(
        game_state.occupied[color as usize]
            & !obstacles
            & !occupied
            & !get_neighbours(game_state.occupied[color.swap() as usize]),
    ))
    .count_ones();
    (12 * free_bee_fields + 4 * bee_moves + our_set_fields) as f64
}
