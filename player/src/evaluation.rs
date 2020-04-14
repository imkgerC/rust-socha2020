use game_sdk::bitboard::get_neighbours;
use game_sdk::gamerules::are_connected_in_swarm;
use game_sdk::{bitboard, get_accessible_neighbors, Color, GameState, PieceType};

pub const COLOR_TO_MOVE: f64 = 12.0;
pub fn evaluate(game_state: &GameState) -> i16 {
    if game_state.ply < 55 {
        (evaluate_color(game_state, Color::RED) - evaluate_color(game_state, Color::BLUE)).round()
            as i16
    } else {
        let red_free =
            (get_neighbours(game_state.pieces[PieceType::BEE as usize][Color::RED as usize])
                & !game_state.occupied()
                & !game_state.obstacles)
                .count_ones();
        let blue_free =
            (get_neighbours(game_state.pieces[PieceType::BEE as usize][Color::BLUE as usize])
                & !game_state.occupied()
                & !game_state.obstacles)
                .count_ones();
        (red_free as i16 - blue_free as i16) * 500
    }
}
pub fn evaluate_color(game_state: &GameState, color: Color) -> f64 {
    let unskewed_phase = game_state.ply as f64 / 60.0;
    let phase = 1.0 - (1.0 - unskewed_phase).powf(2.0);

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
    let our_set = get_neighbours(game_state.occupied[color as usize])
        & !obstacles
        & !occupied
        & !get_neighbours(game_state.occupied[color.swap() as usize]);
    let our_set_next_to_bee = (our_set
        & get_neighbours(game_state.pieces[PieceType::BEE as usize][color.swap() as usize]))
    .count_ones() as f64;
    let our_set_fields = our_set.count_ones() as f64;
    let beetle_on_bee = if bee_index <= 120
        && game_state.is_on_stack(bee_index)
        && (game_state.occupied[color.swap() as usize] & bee) > 0
    {
        1.
    } else {
        0.
    };
    let mut ant_pinning_enemies = 0.;
    let mut ants = game_state.pieces[PieceType::ANT as usize][color as usize];
    while ants > 0 {
        let ant = ants.trailing_zeros();
        if (get_neighbours(1u128 << ant) & occupied).count_ones() == 1
            && get_neighbours(1u128 << ant) & game_state.occupied[color.swap() as usize] > 0
        {
            ant_pinning_enemies += 1.;
        }
        ants ^= 1u128 << ant;
    }
    let mut pinned_pieces = 0.;
    for pt in [
        PieceType::BEE,
        PieceType::BEETLE,
        PieceType::ANT,
        PieceType::SPIDER,
        PieceType::GRASSHOPPER,
    ]
    .iter()
    {
        let mut pieces = game_state.pieces[*pt as usize][color as usize];
        while pieces > 0 {
            let piece_index = pieces.trailing_zeros();
            if !can_be_removed(1u128 << piece_index, occupied) {
                pinned_pieces += 1.;
            }
            pieces ^= 1u128 << piece_index;
        }
    }
    let mut res = 0.;
    res += 12. * free_bee_fields + 4. * bee_moves + our_set_fields - 30. * beetle_on_bee
        + 6. * ant_pinning_enemies
        - 6. * pinned_pieces;
    res += if game_state.color_to_move == color {
        COLOR_TO_MOVE
    } else {
        0.
    };
    res
}

pub fn can_be_removed(from: u128, occupied: u128) -> bool {
    // check if field can be removed and swarm is still connected
    let occupied = occupied ^ from;
    let neighbours = bitboard::get_neighbours(from) & occupied;
    are_connected_in_swarm(occupied, neighbours)
}
