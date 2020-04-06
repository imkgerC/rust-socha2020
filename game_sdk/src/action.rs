use super::piece_type::PieceType;

#[derive(Copy, Clone, PartialEq)]
pub enum Action {
    SkipMove,
    SetMove(PieceType, u8),
    DragMove(PieceType, u8, u8),
}
