use super::bitboard;
use crate::bitboard::constants::VALID_FIELDS;
use crate::{GameState, HashKeys};
use rand::RngCore;

pub const NEIGHBOR_MAGIC: u32 = 134481028u32;
pub const NEIGHBOR_MAGIC_LOOKUP: [u32; 64] = [
    0u32,
    8388610u32,
    2050u32,
    8390656u32,
    8193u32,
    8388609u32,
    10240u32,
    8390656u32,
    8388609u32,
    3u32,
    8388610u32,
    0u32,
    8396800u32,
    0u32,
    8396800u32,
    0u32,
    16779264u32,
    2050u32,
    16777218u32,
    0u32,
    16787457u32,
    2049u32,
    16785408u32,
    0u32,
    16777217u32,
    3u32,
    16777218u32,
    0u32,
    16785408u32,
    0u32,
    16785408u32,
    0u32,
    16777218u32,
    8396800u32,
    16779264u32,
    8398850u32,
    16777217u32,
    8388609u32,
    16779264u32,
    8390656u32,
    25165827u32,
    8193u32,
    25165824u32,
    8194u32,
    25165824u32,
    0u32,
    25165824u32,
    0u32,
    2050u32,
    10240u32,
    0u32,
    8194u32,
    2049u32,
    2049u32,
    0u32,
    0u32,
    3u32,
    8193u32,
    0u32,
    8194u32,
    0u32,
    0u32,
    0u32,
    0u32,
];

pub const NEIGHBOR_MASK_12: u128 = bitboard::get_neighbours(1u128 << CENTERED_FIELD);
const BITS: usize = 6;
const MAX_SIZE: usize = 1 << BITS;
const CENTERED_FIELD: usize = 12;
//Make sure bb is centered for field 12 and anded with Neighbormask12 already
pub fn apply_magic(magic: u32, bb: u32) -> usize {
    (bb.wrapping_mul(magic) >> (32 - BITS)) as usize
}
pub fn is_valid_magic(magic: u32) -> (bool, Option<[u32; MAX_SIZE]>) {
    let invalid_value = std::u32::MAX;
    let mut lookup = [invalid_value; MAX_SIZE];
    for i in 0..MAX_SIZE {
        let bb = put_along_mask(i as u32, NEIGHBOR_MASK_12 as u32);
        let index = apply_magic(magic, bb);
        if lookup[index] == invalid_value {
            lookup[index] =
                get_accessible_neighbors_slow(bb as u128, 0u128, 1u128 << CENTERED_FIELD) as u32;
        } else {
            return (false, None);
        }
    }
    return (true, Some(lookup));
}
pub fn generate_magic() {
    let mut rand = rand::thread_rng();
    loop {
        let random_u32 = rand.next_u32() & rand.next_u32() & rand.next_u32();
        if ((NEIGHBOR_MASK_12 as u32).wrapping_mul(random_u32) & 0xFF00_0000).count_ones() < 5 {
            continue;
        }
        let is_valid = is_valid_magic(random_u32);
        if is_valid.0 {
            println!("pub const NEIGHBOR_MAGIC: u32 = {}u32;\npub const NEIGHBOR_MAGIC_LOOKUP : [u32;64] = {};",random_u32,HashKeys::arr_to_string(&is_valid.1.unwrap(), "u32"));
            break;
        }
    }
}
pub fn put_along_mask(value: u32, mut mask: u32) -> u32 {
    let mut res = 0u32;
    for i in 0..32 {
        let mask_bit = mask.trailing_zeros();
        if (value >> i) & 1 == 1 {
            res |= 1u32 << mask_bit;
        }
        mask ^= 1u32 << mask_bit;
        if mask == 0 {
            break;
        }
    }
    res
}
pub fn get_accessible_neighbors(occupied: u128, obstacles: u128, field: u128) -> u128 {
    let field_index = field.trailing_zeros();
    let (shifted_occ, shifted_ob) = if field_index < 12 {
        (
            occupied << (12 - field_index),
            obstacles << (12 - field_index),
        )
    } else {
        (
            occupied >> (field_index - 12),
            obstacles >> (field_index - 12),
        )
    };
    let res = if shifted_ob & NEIGHBOR_MASK_12 > 0 {
        get_accessible_neighbors_slow(occupied, obstacles, field)
    } else {
        let neighbors = NEIGHBOR_MAGIC_LOOKUP
            [apply_magic(NEIGHBOR_MAGIC, (shifted_occ & NEIGHBOR_MASK_12) as u32)]
            as u128;
        if field_index < 12 {
            (neighbors >> (12 - field_index)) & VALID_FIELDS
        } else {
            (neighbors << (field_index - 12)) & VALID_FIELDS
        }
    };
    res
}
pub fn get_accessible_neighbors_slow(occupied: u128, obstacles: u128, field: u128) -> u128 {
    let free = !(occupied | obstacles);
    let mut ret = 0;
    let nowe = bitboard::shift_nowe(field);
    let noea = bitboard::shift_noea(field);
    let sowe = bitboard::shift_sowe(field);
    let soea = bitboard::shift_soea(field);
    let east = bitboard::shift_east(field);
    let west = bitboard::shift_west(field);
    // check nowe
    let nowe_check = west | noea;
    if nowe_check & obstacles == 0 && (nowe_check & occupied).count_ones() == 1 {
        ret |= nowe;
    }
    // check west
    let west_check = nowe | sowe;
    if west_check & obstacles == 0 && (west_check & occupied).count_ones() == 1 {
        ret |= west;
    }
    // check noea
    let noea_check = nowe | east;
    if noea_check & obstacles == 0 && (noea_check & occupied).count_ones() == 1 {
        ret |= noea;
    }
    // check east
    let east_check = noea | soea;
    if east_check & obstacles == 0 && (east_check & occupied).count_ones() == 1 {
        ret |= east;
    }
    // check sowe
    let sowe_check = soea | west;
    if sowe_check & obstacles == 0 && (sowe_check & occupied).count_ones() == 1 {
        ret |= sowe;
    }
    // check soea
    let soea_check = sowe | east;
    if soea_check & obstacles == 0 && (soea_check & occupied).count_ones() == 1 {
        ret |= soea;
    }

    return ret & free;
}
