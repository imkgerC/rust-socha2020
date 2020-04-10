use game_sdk::bitboard::get_neighbours;
use game_sdk::gamerules::are_connected_in_swarm;
use game_sdk::{bitboard, get_accessible_neighbors, Color, GameState, PieceType};

pub const COLOR_TO_MOVE: f64 = 12.0;
pub fn evaluate(game_state: &GameState) -> i16 {
    let occ = game_state.occupied();
    let pinners = game_state.get_pinners();
    let mut pinned_pieces = get_neighbours(pinners) & occ;
    let mut already_checked = pinners;
    let mut it = 0u128;
    while it & !already_checked > 0 {
        let index = (it & !already_checked).trailing_zeros() as usize;
        let mut possible_pinned = get_neighbours(1u128 << index) & !already_checked & occ;
        already_checked |= 1u128 << index;
        while possible_pinned > 0 {
            let i = possible_pinned.trailing_zeros() as usize;
            possible_pinned ^= 1u128 << i;
            let neighbors = get_neighbours(1u128 << i) & occ;
            if neighbors.count_ones() == 2 {
                let n1 = neighbors.trailing_zeros();
                let neighborsn1 = get_neighbours(1u128 << n1) & neighbors;
                if neighborsn1 == 0 {
                    pinned_pieces |= 1u128 << i;
                    it |= 1u128 << i;
                    continue;
                }
            }
            already_checked |= 1u128 << i;
        }
    }
    (evaluate_color(game_state, Color::RED, pinners, pinned_pieces)
        - evaluate_color(game_state, Color::BLUE, pinners, pinned_pieces))
    .round() as i16
}

#[inline(always)]
pub fn evaluate_color(game_state: &GameState, color: Color, pinners: u128, pinned: u128) -> f64 {
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
    let beetle_on_bee = if bee_index <= 120
        && game_state.is_on_stack(bee_index)
        && (game_state.occupied[color.swap() as usize] & bee) > 0
    {
        1.
    } else {
        0.
    };
    let pins_enemies =
        get_neighbours(get_neighbours(pinners) & game_state.occupied[color.swap() as usize])
            & pinners;
    let ant_pinning_enemies = (pins_enemies
        & game_state.pieces[PieceType::ANT as usize][color as usize])
        .count_ones() as f64;
    //let spider_pinning_enemies = (pins_enemies & game_state.pieces[PieceType::SPIDER as usize][color as usize]).count_ones() as f64;
    //let grasshopper_pinning_enemies = (pins_enemies & game_state.pieces[PieceType::GRASSHOPPER as usize][color as usize]).count_ones() as f64;
    let pinned_pieces = (pinned & game_state.pieces_from_color(color)).count_ones() as f64;
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

#[inline(always)]
pub fn can_be_removed(from: u128, occupied: u128) -> bool {
    // check if field can be removed and swarm is still connected
    let occupied = occupied ^ from;
    let neighbours = bitboard::get_neighbours(from) & occupied;
    are_connected_in_swarm(occupied, neighbours)
}
