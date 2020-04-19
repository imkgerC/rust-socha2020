use game_sdk::bitboard::get_neighbours;
use game_sdk::gamerules::are_connected_in_swarm;
use game_sdk::{bitboard, get_accessible_neighbors, Color, GameState, PieceType};

pub struct EvaluationParameters {
    pub tempo_bonus: f64,
    pub pinned_factor: f64,
    pub ant_pin_factor: f64,
    pub bee_move_factor: f64,
    pub beetle_factor: f64,
    pub free_factor: f64,
    pub set_fields_factor: f64,
    pub free_factor_phased: f64,
    pub free_own_beetle: f64,
    pub free_own: f64,
}

impl EvaluationParameters {
    pub const fn from_array(params: [f64; 10]) -> EvaluationParameters {
        EvaluationParameters {
            tempo_bonus: params[0],
            pinned_factor: params[1],
            ant_pin_factor: params[2],
            bee_move_factor: params[3],
            beetle_factor: params[4],
            free_factor: params[5],
            set_fields_factor: params[6],
            free_factor_phased: params[7],
            free_own_beetle: params[8],
            free_own: params[9],
        }
    }
}

pub const COLOR_TO_MOVE: f64 = 12.0;

// pub const DEFAULT_ARRAY: [f64; 10] = [12.0, -6.0, 6.0, 4.0, -30.0, 12.0, 1.0, 24.0, 0.33, 0.25];
/*pub const DEFAULT_ARRAY: [f64; 10] = [
    7.338336897141312,
    -22.784990014994456,
    -9.098141752046942,
    0.2718628254902993,
    -67.27154376977042,
    0.3289252083147839,
    3.9327962306265025,
    110.27935167257316,
    -0.01999999999999995,
    0.39999999999999997,
];*/
pub const DEFAULT_ARRAY: [f64; 10] = [
    4.465854911107787,
    -17.11870023666,
    0.6434493074867715,
    0.6604163953067962,
    -58.461513000000004,
    0.8614775852302234,
    2.5167057152925705,
    75.322281041304,
    -0.01999999999999995,
    0.49999999999999994,
];

pub const DEFAULT: EvaluationParameters = EvaluationParameters::from_array(DEFAULT_ARRAY);

pub fn evaluate(game_state: &GameState) -> i16 {
    (evaluate_color(game_state, Color::RED, &DEFAULT)
        - evaluate_color(game_state, Color::BLUE, &DEFAULT))
    .round() as i16
}

pub fn evaluate_with_parameters(game_state: &GameState, params: &EvaluationParameters) -> i16 {
    (evaluate_color(game_state, Color::RED, params)
        - evaluate_color(game_state, Color::BLUE, params))
    .round() as i16
}

pub fn evaluate_color(game_state: &GameState, color: Color, params: &EvaluationParameters) -> f64 {
    let occupied = game_state.occupied();
    let obstacles = game_state.obstacles;

    let bee = game_state.pieces[PieceType::BEE as usize][color as usize];
    let bee_index = bee.trailing_zeros() as usize;
    let bee_neighbors = get_neighbours(bee);
    let bee_moves = get_accessible_neighbors(occupied, obstacles, bee).count_ones() as f64;
    let mut free_bee_fields = (bee_neighbors & !occupied & !obstacles).count_ones() as f64;
    free_bee_fields += params.free_own
        * (bee_neighbors & game_state.occupied[color as usize]).count_ones() as f64
        + params.free_own_beetle
            * (bee_neighbors & game_state.pieces[PieceType::BEETLE as usize][color as usize])
                .count_ones() as f64;
    let our_set = get_neighbours(game_state.occupied[color as usize])
        & !obstacles
        & !occupied
        & !get_neighbours(game_state.occupied[color.swap() as usize]);
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
            let piece_bit = 1 << piece_index;
            pieces ^= piece_bit;
            if !can_be_removed(piece_bit, occupied) {
                pinned_pieces += 1.;
            }
        }
    }

    let mut res = 0.;
    res += params.free_factor * free_bee_fields
        + params.bee_move_factor * bee_moves
        + params.set_fields_factor * our_set_fields
        + params.beetle_factor * beetle_on_bee
        + params.ant_pin_factor * ant_pinning_enemies
        + params.free_factor_phased * (game_state.ply as f64 / 60.) * free_bee_fields
        + params.pinned_factor * pinned_pieces;
    res += if game_state.color_to_move == color {
        params.tempo_bonus
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
