use crate::piece_type::PieceType;
#[derive(Copy, Clone, Debug)]
pub enum FieldType {
    BLOCKED,
    FREE,
    USED(PieceType),
}
impl FieldType {
    pub fn to_string(&self) -> String {
        match self {
            FieldType::BLOCKED => "X".to_owned(),
            FieldType::FREE => " ".to_owned(),
            FieldType::USED(pt) => pt.to_string(),
        }
    }
}
