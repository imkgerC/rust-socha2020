#![allow(dead_code)]

pub mod constants {
    pub const VALID_FIELDS: u128 = 2618206181369836630229216686713862207;
    pub const SHIFT_NOWE_MASK: u128 = 2616927760382839639705131374633549824;
    pub const SHIFT_NOEA_MASK: u128 = 2617566970876338134958173432248795136;
    pub const SHIFT_SOWE_MASK: u128 = 36046397799141345;
    pub const SHIFT_SOEA_MASK: u128 = 36902497546234103776;
    pub const SHIFT_EAST_MASK: u128 = 1329877349959700883091619741417209888;
    pub const SHIFT_WEST_MASK: u128 = 41548518549565135954022796120557569;
    pub const SHIFT_EAST_UNSAFE_MASK: u128 = 1329877349959700883082610342602211328;
    pub const SHIFT_WEST_UNSAFE_MASK: u128 = 36046397799139329;

    pub const SHIFT_WEST_TO_EAST_MASK: u128 = ((!SHIFT_EAST_MASK << 1) & !SHIFT_EAST_MASK) << 1;
    pub const SHIFT_WEST_TO_NOEA_MASK: u128 = ((!SHIFT_EAST_MASK << 1) & !SHIFT_NOEA_MASK) << 12;
    pub const SHIFT_WEST_TO_NOWE_MASK: u128 = ((!SHIFT_EAST_MASK << 1) & !SHIFT_NOWE_MASK) << 11;
    pub const SHIFT_WEST_TO_SOEA_MASK: u128 = ((!SHIFT_EAST_MASK << 1) & !SHIFT_SOEA_MASK) >> 11;
    pub const SHIFT_WEST_TO_SOWE_MASK: u128 = ((!SHIFT_EAST_MASK << 1) & !SHIFT_SOWE_MASK) >> 12;
    pub const SHIFT_EAST_TO_WEST_MASK: u128 = ((!SHIFT_WEST_MASK >> 1) & !SHIFT_WEST_MASK) >> 1;
    pub const SHIFT_EAST_TO_NOEA_MASK: u128 = ((!SHIFT_WEST_MASK >> 1) & !SHIFT_NOEA_MASK) << 12;
    pub const SHIFT_EAST_TO_NOWE_MASK: u128 = ((!SHIFT_WEST_MASK >> 1) & !SHIFT_NOWE_MASK) << 11;
    pub const SHIFT_EAST_TO_SOEA_MASK: u128 = ((!SHIFT_WEST_MASK >> 1) & !SHIFT_SOEA_MASK) >> 11;
    pub const SHIFT_EAST_TO_SOWE_MASK: u128 = ((!SHIFT_WEST_MASK >> 1) & !SHIFT_SOWE_MASK) >> 12;
    pub const SHIFT_NOEA_TO_WEST_MASK: u128 = ((!SHIFT_SOWE_MASK >> 12) & !SHIFT_WEST_MASK) >> 1;
    pub const SHIFT_NOEA_TO_EAST_MASK: u128 = ((!SHIFT_SOWE_MASK >> 12) & !SHIFT_EAST_MASK) << 1;
    pub const SHIFT_NOEA_TO_NOWE_MASK: u128 = ((!SHIFT_SOWE_MASK >> 12) & !SHIFT_NOWE_MASK) << 11;
    pub const SHIFT_NOEA_TO_SOEA_MASK: u128 = ((!SHIFT_SOWE_MASK >> 12) & !SHIFT_SOEA_MASK) >> 11;
    pub const SHIFT_NOEA_TO_SOWE_MASK: u128 = ((!SHIFT_SOWE_MASK >> 12) & !SHIFT_SOWE_MASK) >> 12;
    pub const SHIFT_NOWE_TO_WEST_MASK: u128 = ((!SHIFT_SOEA_MASK >> 11) & !SHIFT_WEST_MASK) >> 1;
    pub const SHIFT_NOWE_TO_EAST_MASK: u128 = ((!SHIFT_SOEA_MASK >> 11) & !SHIFT_EAST_MASK) << 1;
    pub const SHIFT_NOWE_TO_NOEA_MASK: u128 = ((!SHIFT_SOEA_MASK >> 11) & !SHIFT_NOEA_MASK) << 12;
    pub const SHIFT_NOWE_TO_SOEA_MASK: u128 = ((!SHIFT_SOEA_MASK >> 11) & !SHIFT_SOEA_MASK) >> 11;
    pub const SHIFT_NOWE_TO_SOWE_MASK: u128 = ((!SHIFT_SOEA_MASK >> 11) & !SHIFT_SOWE_MASK) >> 12;
    pub const SHIFT_SOEA_TO_WEST_MASK: u128 = ((!SHIFT_NOWE_MASK << 11) & !SHIFT_WEST_MASK) >> 1;
    pub const SHIFT_SOEA_TO_EAST_MASK: u128 = ((!SHIFT_NOWE_MASK << 11) & !SHIFT_EAST_MASK) << 1;
    pub const SHIFT_SOEA_TO_NOEA_MASK: u128 = ((!SHIFT_NOWE_MASK << 11) & !SHIFT_NOEA_MASK) << 12;
    pub const SHIFT_SOEA_TO_NOWE_MASK: u128 = ((!SHIFT_NOWE_MASK << 11) & !SHIFT_NOWE_MASK) << 11;
    pub const SHIFT_SOEA_TO_SOWE_MASK: u128 = ((!SHIFT_NOWE_MASK << 11) & !SHIFT_SOWE_MASK) >> 12;
    pub const SHIFT_SOWE_TO_WEST_MASK: u128 = ((!SHIFT_NOEA_MASK << 12) & !SHIFT_WEST_MASK) >> 1;
    pub const SHIFT_SOWE_TO_EAST_MASK: u128 = ((!SHIFT_NOEA_MASK << 12) & !SHIFT_EAST_MASK) << 1;
    pub const SHIFT_SOWE_TO_NOEA_MASK: u128 = ((!SHIFT_NOEA_MASK << 12) & !SHIFT_NOEA_MASK) << 12;
    pub const SHIFT_SOWE_TO_NOWE_MASK: u128 = ((!SHIFT_NOEA_MASK << 12) & !SHIFT_NOWE_MASK) << 11;
    pub const SHIFT_SOWE_TO_SOEA_MASK: u128 = ((!SHIFT_NOEA_MASK << 12) & !SHIFT_SOEA_MASK) >> 11;
}

#[inline(always)]
pub const fn get_neighbours(bitboard: u128) -> u128 {
    let shifted_east = shift_east_unsafe(bitboard) | bitboard;
    let right_portion = shift_nowe(shifted_east) | shifted_east;
    let shifted_west = shift_west_unsafe(bitboard) | bitboard;
    let left_portion = shift_soea(shifted_west) | shifted_west;
    return ((left_portion | right_portion) ^ bitboard) & constants::VALID_FIELDS;
}

#[inline(always)]
pub const fn shift_east(bitboard: u128) -> u128 {
    return (bitboard & !constants::SHIFT_EAST_MASK) << 1;
}

#[inline(always)]
pub const fn shift_west(bitboard: u128) -> u128 {
    return (bitboard & !constants::SHIFT_WEST_MASK) >> 1;
}

#[inline(always)]
pub const fn shift_east_unsafe(bitboard: u128) -> u128 {
    return (bitboard & !constants::SHIFT_EAST_UNSAFE_MASK) << 1;
}

#[inline(always)]
pub const fn shift_west_unsafe(bitboard: u128) -> u128 {
    return (bitboard & !constants::SHIFT_WEST_UNSAFE_MASK) >> 1;
}

#[inline(always)]
pub const fn shift_nowe(bitboard: u128) -> u128 {
    return (bitboard & !constants::SHIFT_NOWE_MASK) << 11;
}

#[inline(always)]
pub const fn shift_soea(bitboard: u128) -> u128 {
    return (bitboard & !constants::SHIFT_SOEA_MASK) >> 11;
}

#[inline(always)]
pub const fn shift_sowe(bitboard: u128) -> u128 {
    return (bitboard & !constants::SHIFT_SOWE_MASK) >> 12;
}

#[inline(always)]
pub const fn shift_noea(bitboard: u128) -> u128 {
    return (bitboard & !constants::SHIFT_NOEA_MASK) << 12;
}

#[inline(always)]
pub const fn shift_west_to_east(bitboard: u128) -> u128 {
    return (bitboard << 2) & constants::SHIFT_WEST_TO_EAST_MASK;
}
#[inline(always)]
pub const fn shift_west_to_noea(bitboard: u128) -> u128 {
    return (bitboard << 13) & constants::SHIFT_WEST_TO_NOEA_MASK;
}
#[inline(always)]
pub const fn shift_west_to_nowe(bitboard: u128) -> u128 {
    return (bitboard << 12) & constants::SHIFT_WEST_TO_NOWE_MASK;
}
#[inline(always)]
pub const fn shift_west_to_soea(bitboard: u128) -> u128 {
    return (bitboard >> 10) & constants::SHIFT_WEST_TO_SOEA_MASK;
}
#[inline(always)]
pub const fn shift_west_to_sowe(bitboard: u128) -> u128 {
    return (bitboard >> 11) & constants::SHIFT_WEST_TO_SOWE_MASK;
}
#[inline(always)]
pub const fn shift_east_to_west(bitboard: u128) -> u128 {
    return (bitboard >> 2) & constants::SHIFT_EAST_TO_WEST_MASK;
}
#[inline(always)]
pub const fn shift_east_to_noea(bitboard: u128) -> u128 {
    return (bitboard << 11) & constants::SHIFT_EAST_TO_NOEA_MASK;
}
#[inline(always)]
pub const fn shift_east_to_nowe(bitboard: u128) -> u128 {
    return (bitboard << 10) & constants::SHIFT_EAST_TO_NOWE_MASK;
}
#[inline(always)]
pub const fn shift_east_to_soea(bitboard: u128) -> u128 {
    return (bitboard >> 12) & constants::SHIFT_EAST_TO_SOEA_MASK;
}
#[inline(always)]
pub const fn shift_east_to_sowe(bitboard: u128) -> u128 {
    return (bitboard >> 13) & constants::SHIFT_EAST_TO_SOWE_MASK;
}
#[inline(always)]
pub const fn shift_noea_to_west(bitboard: u128) -> u128 {
    return (bitboard >> 13) & constants::SHIFT_NOEA_TO_WEST_MASK;
}
#[inline(always)]
pub const fn shift_noea_to_east(bitboard: u128) -> u128 {
    return (bitboard >> 11) & constants::SHIFT_NOEA_TO_EAST_MASK;
}
#[inline(always)]
pub const fn shift_noea_to_nowe(bitboard: u128) -> u128 {
    return (bitboard >> 1) & constants::SHIFT_NOEA_TO_NOWE_MASK;
}
#[inline(always)]
pub const fn shift_noea_to_soea(bitboard: u128) -> u128 {
    return (bitboard >> 23) & constants::SHIFT_NOEA_TO_SOEA_MASK;
}
#[inline(always)]
pub const fn shift_noea_to_sowe(bitboard: u128) -> u128 {
    return (bitboard >> 24) & constants::SHIFT_NOEA_TO_SOWE_MASK;
}
#[inline(always)]
pub const fn shift_nowe_to_west(bitboard: u128) -> u128 {
    return (bitboard >> 12) & constants::SHIFT_NOWE_TO_WEST_MASK;
}
#[inline(always)]
pub const fn shift_nowe_to_east(bitboard: u128) -> u128 {
    return (bitboard >> 10) & constants::SHIFT_NOWE_TO_EAST_MASK;
}
#[inline(always)]
pub const fn shift_nowe_to_noea(bitboard: u128) -> u128 {
    return (bitboard << 1) & constants::SHIFT_NOWE_TO_NOEA_MASK;
}
#[inline(always)]
pub const fn shift_nowe_to_soea(bitboard: u128) -> u128 {
    return (bitboard >> 22) & constants::SHIFT_NOWE_TO_SOEA_MASK;
}
#[inline(always)]
pub const fn shift_nowe_to_sowe(bitboard: u128) -> u128 {
    return (bitboard >> 23) & constants::SHIFT_NOWE_TO_SOWE_MASK;
}
#[inline(always)]
pub const fn shift_soea_to_west(bitboard: u128) -> u128 {
    return (bitboard << 10) & constants::SHIFT_SOEA_TO_WEST_MASK;
}
#[inline(always)]
pub const fn shift_soea_to_east(bitboard: u128) -> u128 {
    return (bitboard << 12) & constants::SHIFT_SOEA_TO_EAST_MASK;
}
#[inline(always)]
pub const fn shift_soea_to_noea(bitboard: u128) -> u128 {
    return (bitboard << 23) & constants::SHIFT_SOEA_TO_NOEA_MASK;
}
#[inline(always)]
pub const fn shift_soea_to_nowe(bitboard: u128) -> u128 {
    return (bitboard << 22) & constants::SHIFT_SOEA_TO_NOWE_MASK;
}
#[inline(always)]
pub const fn shift_soea_to_sowe(bitboard: u128) -> u128 {
    return (bitboard >> 1) & constants::SHIFT_SOEA_TO_SOWE_MASK;
}
#[inline(always)]
pub const fn shift_sowe_to_west(bitboard: u128) -> u128 {
    return (bitboard << 11) & constants::SHIFT_SOWE_TO_WEST_MASK;
}
#[inline(always)]
pub const fn shift_sowe_to_east(bitboard: u128) -> u128 {
    return (bitboard << 13) & constants::SHIFT_SOWE_TO_EAST_MASK;
}
#[inline(always)]
pub const fn shift_sowe_to_noea(bitboard: u128) -> u128 {
    return (bitboard << 24) & constants::SHIFT_SOWE_TO_NOEA_MASK;
}
#[inline(always)]
pub const fn shift_sowe_to_nowe(bitboard: u128) -> u128 {
    return (bitboard << 23) & constants::SHIFT_SOWE_TO_NOWE_MASK;
}
#[inline(always)]
pub const fn shift_sowe_to_soea(bitboard: u128) -> u128 {
    return (bitboard << 1) & constants::SHIFT_SOWE_TO_SOEA_MASK;
}
