pub mod constants {
    pub static VALID_FIELDS: u128 = 2618206181369836630229216686713862207;
    pub static SHIFT_NOWE_MASK: u128 = 2616927760382839639705131374633549824;
    pub static SHIFT_NOEA_MASK: u128 = 2617566970876338134958173432248795136;
    pub static SHIFT_SOWE_MASK: u128 = 36046397799141345;
    pub static SHIFT_SOEA_MASK: u128 = 36902497546234103776;
    pub static SHIFT_EAST_MASK: u128 = 1329877349959700883091619741417209888;
    pub static SHIFT_WEST_MASK: u128 = 41548518549565135954022796120557569;
}

#[inline(always)]
pub fn get_neighbours(bitboard: u128) -> u128 {
    let shifted_east = shift_east(bitboard) | bitboard;
    let right_portion = shift_nowe(shifted_east) | shifted_east;
    let shifted_west = shift_west(bitboard) | bitboard;
    let left_portion = shift_soea(shifted_west) | shifted_west;
    return (left_portion | right_portion) ^ bitboard;
}

#[inline(always)]
pub fn shift_east(bitboard: u128) -> u128 {
    return (bitboard & !constants::SHIFT_EAST_MASK) << 1;
}

#[inline(always)]
pub fn shift_west(bitboard: u128) -> u128 {
    return (bitboard & !constants::SHIFT_WEST_MASK) >> 1;
}

#[inline(always)]
pub fn shift_nowe(bitboard: u128) -> u128 {
    return (bitboard & !constants::SHIFT_NOWE_MASK) << 11;
}

#[inline(always)]
pub fn shift_soea(bitboard: u128) -> u128 {
    return (bitboard & !constants::SHIFT_SOEA_MASK) >> 11;
}

#[inline(always)]
pub fn shift_sowe(bitboard: u128) -> u128 {
    return (bitboard & !constants::SHIFT_SOWE_MASK) >> 12;
}

#[inline(always)]
pub fn shift_noea(bitboard: u128) -> u128 {
    return (bitboard & !constants::SHIFT_NOEA_MASK) << 12;
}
