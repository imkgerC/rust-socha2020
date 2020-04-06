#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PieceType {
    BEE = 0,
    ANT = 1,
    BEETLE = 2,
    GRASSHOPPER = 3,
    SPIDER = 4,
}

pub static VARIANTS: [PieceType; 5] = [
    PieceType::BEE,
    PieceType::BEETLE,
    PieceType::ANT,
    PieceType::GRASSHOPPER,
    PieceType::SPIDER,
];
