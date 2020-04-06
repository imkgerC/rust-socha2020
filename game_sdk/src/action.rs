use super::piece_type::PieceType;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Action {
    SkipMove,
    SetMove(PieceType, u8),
    RegMove(PieceType, u8, u8),
}
