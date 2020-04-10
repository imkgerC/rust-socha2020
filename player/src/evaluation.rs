use game_sdk::bitboard::get_neighbours;
use game_sdk::{get_accessible_neighbors, Color, GameState, PieceType};

pub const COLOR_TO_MOVE: f64 = 12.0;
pub fn evaluate(game_state: &GameState) -> i16 {
    (evaluate_color(game_state, Color::RED) - evaluate_color(game_state, Color::BLUE)).round()
        as i16
}
pub fn evaluate_color(game_state: &GameState, color: Color) -> f64 {
    let occupied = game_state.occupied();
    let obstacles = game_state.obstacles;

    let bee = game_state.pieces[PieceType::BEE as usize][color as usize];
    let bee_index = bee.trailing_zeros() as usize;
    let bee_neighbors = get_neighbours(bee);
    let bee_moves = get_accessible_neighbors(occupied, obstacles, bee).count_ones() as f64;
    let mut free_bee_fields = (bee_neighbors & !occupied & !obstacles).count_ones() as f64;
    free_bee_fields += 0.25
        * (bee_neighbors & game_state.occupied[color as usize]).count_ones() as f64
        + 0.33
            * (bee_neighbors & game_state.pieces[PieceType::BEETLE as usize][color as usize])
                .count_ones() as f64;
    let our_set_fields = (get_neighbours(
        game_state.occupied[color as usize]
            & !obstacles
            & !occupied
            & !get_neighbours(game_state.occupied[color.swap() as usize]),
    ))
    .count_ones() as f64;
    let beetle_on_bee = if game_state.is_on_stack(bee_index)
        && (game_state.occupied[color.swap() as usize] & bee) > 0
    {
        1.
    } else {
        0.
    };
    let mut res = 0.;
    res += (12. * free_bee_fields + 4. * bee_moves + our_set_fields - 30. * beetle_on_bee);
    res += if game_state.color_to_move == color {
        COLOR_TO_MOVE
    } else {
        0.
    };
    res
}
