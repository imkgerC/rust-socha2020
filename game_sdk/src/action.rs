use super::gamestate::Color;
use super::piece_type::PieceType;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Action {
    SkipMove,
    SetMove(PieceType, u8),
    DragMove(PieceType, u8, u8),
}

impl Action {
    pub fn get_xml(&self, color: Color) -> String {
        let mut ret = "".to_string();
        match self {
            Action::SkipMove => {
                ret.push_str("<data class=\"skipmove\" />");
            }
            Action::SetMove(piece_type, to) => {
                ret.push_str("<data class=\"setmove\">");
                let type_string = match piece_type {
                    PieceType::ANT => "ANT",
                    PieceType::BEE => "BEE",
                    PieceType::BEETLE => "BEETLE",
                    PieceType::SPIDER => "SPIDER",
                    PieceType::GRASSHOPPER => "GRASSHOPPER",
                };
                ret.push_str(&format!(
                    "\n  <piece owner=\"{}\" type=\"{}\" />",
                    color.to_string(),
                    type_string
                ));
                let dest_x = (*to as i32 % 11) - 5;
                let dest_z = -((*to as i32 / 11) - 5);
                let dest_y = -dest_z - dest_x;
                ret.push_str(&format!(
                    "\n  <destination x=\"{}\" y=\"{}\" z=\"{}\" />\n",
                    dest_x, dest_y, dest_z
                ));
                ret.push_str("</data>");
            }
            Action::DragMove(_piece_type, from, to) => {
                ret.push_str("<data class=\"dragmove\">");
                let start_x = (*from as i32 % 11) - 5;
                let dest_x = (*to as i32 % 11) - 5;
                let start_z = -((*from as i32 / 11) - 5);
                let dest_z = -((*to as i32 / 11) - 5);
                // x + y + z= 0 <=> y = - z - x
                let start_y = -start_z - start_x;
                let dest_y = -dest_z - dest_x;
                ret.push_str(&format!(
                    "<start x=\"{}\" y=\"{}\" z=\"{}\" />",
                    start_x, start_y, start_z
                ));
                ret.push_str(&format!(
                    "<destination x=\"{}\" y=\"{}\" z=\"{}\" />",
                    dest_x, dest_y, dest_z
                ));
                ret.push_str("</data>");
            }
        };
        return ret;
    }

    pub fn to_string(&self) -> String {
        match self {
            Action::SkipMove => "SkipMove".to_owned(),
            Action::SetMove(pt, to) => format!("SetMove {} {}", pt.to_string(), to),
            Action::DragMove(pt, from, to) => {
                format!("DragMove {} {} {}", pt.to_string(), from, to)
            }
        }
    }

    pub fn from_string(str: String) -> Self {
        let descs: Vec<&str> = str.split(" ").collect();
        assert!(!descs.is_empty());
        match descs[0].to_uppercase().as_str() {
            "SKIPMOVE" => Action::SkipMove,
            "SETMOVE" => {
                assert_eq!(descs.len(), 3);
                let pt = PieceType::from_string(descs[1].to_owned());
                let to = descs[2].parse::<u8>().expect("Invalid to field");
                Action::SetMove(pt, to)
            }
            "DRAGMOVE" => {
                assert_eq!(descs.len(), 4);
                let pt = PieceType::from_string(descs[1].to_owned());
                let from = descs[2].parse::<u8>().expect("Invalid from field");
                let to = descs[3].parse::<u8>().expect("Invalid to field");
                Action::DragMove(pt, from, to)
            }
            _ => panic!("Invalid action description"),
        }
    }
}
