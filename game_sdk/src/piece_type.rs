#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PieceType {
    BEE = 0,
    ANT = 1,
    BEETLE = 2,
    GRASSHOPPER = 3,
    SPIDER = 4,
}
impl PieceType {
    pub fn to_string(&self) -> String {
        match self {
            PieceType::BEE => "Q".to_owned(),
            PieceType::ANT => "A".to_owned(),
            PieceType::BEETLE => "B".to_owned(),
            PieceType::GRASSHOPPER => "G".to_owned(),
            PieceType::SPIDER => "S".to_owned(),
        }
    }
    pub fn from_string(str: String) -> Self {
        match str.to_uppercase().as_str() {
            "Q" => PieceType::BEE,
            "A" => PieceType::ANT,
            "B" => PieceType::BEETLE,
            "G" => PieceType::GRASSHOPPER,
            "S" => PieceType::SPIDER,
            _ => panic!("Invalid piece type description"),
        }
    }
}

pub static PIECETYPE_VARIANTS: [PieceType; 5] = [
    PieceType::BEE,
    PieceType::BEETLE,
    PieceType::ANT,
    PieceType::GRASSHOPPER,
    PieceType::SPIDER,
];
