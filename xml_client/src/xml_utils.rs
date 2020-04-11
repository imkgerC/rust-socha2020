extern crate game_sdk;
extern crate xml;

use self::game_sdk::*;
use self::xml::reader::{EventReader, XmlEvent};
use std::collections::HashMap;
use std::collections::VecDeque;
use std::io::BufReader;
use std::net::TcpStream;
use std::vec::Vec;

#[derive(Debug)]
pub struct XMLNode {
    name: String,
    data: String,
    attribs: HashMap<String, Vec<String>>,
    childs: Vec<XMLNode>,
}

impl XMLNode {
    fn new() -> XMLNode {
        return XMLNode {
            name: String::new(),
            data: String::new(),
            attribs: HashMap::new(),
            childs: Vec::new(),
        };
    }

    pub fn read_from(xml_parser: &mut EventReader<BufReader<&TcpStream>>) -> XMLNode {
        let mut node_stack: VecDeque<XMLNode> = VecDeque::new();
        let mut has_received_first = false;
        let mut final_node: Option<XMLNode> = None;

        loop {
            match xml_parser.next() {
                Ok(XmlEvent::StartElement {
                    name, attributes, ..
                }) => {
                    let mut node = XMLNode::new();
                    node.name = name.local_name;
                    for attribute in attributes {
                        let attrib_name = attribute.name.local_name;
                        if !node.attribs.contains_key(&attrib_name) {
                            node.attribs.insert(attrib_name.to_string(), Vec::new());
                        }
                        node.attribs
                            .get_mut(&attrib_name)
                            .unwrap()
                            .push(attribute.value.to_string());
                    }
                    node_stack.push_back(node);
                    has_received_first = true;
                }
                Ok(XmlEvent::EndElement { .. }) => {
                    if node_stack.len() > 2 {
                        let child = node_stack.pop_back().expect("Unexpectedly found empty XML node stack while trying to pop off new child element");
                        let mut node = node_stack.pop_back().expect("Unexpectedly found empty XML node stack while trying to hook up new child element");
                        node.childs.push(child);
                        node_stack.push_back(node);
                    } else if has_received_first {
                        final_node = Some(node_stack.pop_back().expect(
                            "Unexpectedly found empty XML node stack while trying to return node",
                        ));
                    }
                }
                Ok(XmlEvent::Characters(content)) => {
                    node_stack.back_mut().expect("Unexpectedly found empty XML node stack while trying to add characters").data += content.as_str();
                }
                Err(_) => {
                    break;
                }
                _ => {}
            }

            // Exit condition
            if final_node.is_some() {
                break;
            }
        }

        return final_node.unwrap(); // Is guaranteed to be present due to the condition above
    }

    pub fn as_game_state(&self) -> GameState {
        let err = "Error while parsing XML node to GameState";
        let mut state = GameState::new();
        state.ply = self
            .get_attribute("turn")
            .expect(err)
            .parse::<u8>()
            .expect(err);
        let (occupied, obstacles, pieces, beetle_stack) =
            self.get_child("board").expect(err).as_board();
        state.color_to_move = if state.ply % 2 == 0 {
            Color::RED
        } else {
            Color::BLUE
        };
        state.occupied = occupied;
        state.obstacles = obstacles;
        state.pieces = pieces;
        state.beetle_stack = beetle_stack;
        state.hash = GameState::calculate_hash(
            &state.pieces,
            state.color_to_move,
            &state.beetle_stack,
            state.ply,
        );
        state.recalculate_undeployed();
        return state;
    }

    /* pub fn as_player(&self) -> Player {
        let err = "Error while parsing XML node to Player";
        return Player {
            display_name: self.get_attribute("displayName").expect(err).to_string(),
            color: match self.get_attribute("color").expect(err).to_string().as_str() {
                "RED" => PlayerColor::Red,
                "BLUE" => PlayerColor::Blue,
                _ => panic!("Error parsing Player"),
            },
        };
    } */

    pub fn as_room(&self) -> String {
        let err = "Error while parsing XML node to Room";
        return self.get_attribute("roomId").expect(err).to_string();
    }

    /*pub fn _as_joined(&self) -> Joined {
        let err = "Error while parsing XML node to Joined";
        return Joined {
            id: self.get_attribute("roomId").expect(err).to_string(),
        };
    }*/

    #[allow(unused)]
    pub fn winner_string(&self) -> String {
        let _err = "Error while parsing XML node to WinnerString";
        let winner_str;
        if let Some(winner) = self.get_child("winner") {
            winner_str = format!(
                "{} wins as {}",
                winner
                    .get_attribute("displayName")
                    .unwrap_or(&"Draw".to_string()),
                winner.get_attribute("color").unwrap_or(&"Draw".to_string())
            );
        } else {
            winner_str = format!("DRAW");
        }
        let cause_str = self
            .get_child("score")
            .expect("did not find score while parsing XML node")
            .get_attribute("reason")
            .expect("did not find reason while parsing XML node");
        return format!("{}, reason: {}", winner_str, cause_str);
    }

    pub fn as_welcome_message(&self) -> Color {
        let err = "Error while parsing XML node to WelcomeMessage";
        match self.get_attribute("color").expect(err).as_str() {
            "red" => Color::RED,
            "blue" => Color::BLUE,
            _ => panic!(err),
        }
    }

    pub fn as_board(&self) -> ([u128; 2], u128, [[u128; 2]; 5], [[u128; 2]; 4]) {
        let mut occupied = [0; 2];
        let mut obstacles = 0;
        let mut pieces = [[0; 2]; 5];
        let mut beetle_stack = [[0; 2]; 4];
        for row in self.get_child_vec("fields").iter() {
            for (shift, is_obstacle, lowest_piece, lowest_color, color, beetles) in
                row.get_child_vec("field").iter().map(|n| n.as_field())
            {
                if is_obstacle {
                    obstacles |= 1 << shift;
                    continue;
                }
                if let Some(lowest_piece) = lowest_piece {
                    if let Some(lowest_color) = lowest_color {
                        pieces[lowest_piece as usize][lowest_color as usize] |= 1 << shift;
                    } else {
                        panic!("did not find color of lowest piece");
                    }
                }
                if let Some(color) = color {
                    occupied[color as usize] |= 1 << shift;
                }
                for (idx, color) in beetles.into_iter().enumerate() {
                    beetle_stack[idx][color as usize] |= 1 << shift;
                }
            }
        }
        return (occupied, obstacles, pieces, beetle_stack);
    }

    pub fn as_memento(&self) -> GameState {
        let err = "Error while parsing XML node to Memento";
        return self.get_child("state").expect(err).as_game_state();
    }

    pub fn as_field(
        &self,
    ) -> (
        u8,
        bool,
        Option<PieceType>,
        Option<Color>,
        Option<Color>,
        Vec<Color>,
    ) {
        let err = "Error while parsing XML node to Field";
        let their_x = self
            .get_attribute("x")
            .expect(err)
            .parse::<i8>()
            .expect(err);
        let their_z = self
            .get_attribute("z")
            .expect(err)
            .parse::<i8>()
            .expect(err);
        let our_x = (their_x + 5) as u8;
        let our_z = (-their_z + 5) as u8;
        let shift = our_x + 11 * our_z;
        let is_obstacle = self
            .get_attribute("isObstructed")
            .expect(err)
            .parse::<bool>()
            .expect(err);
        if is_obstacle {
            return (shift, is_obstacle, None, None, None, Vec::new());
        }
        let mut pieces_xml = self.get_child_vec("piece");
        if pieces_xml.len() == 0 {
            return (shift, is_obstacle, None, None, None, Vec::new());
        }
        if pieces_xml.len() == 1 {
            let only_piece = pieces_xml.get(0).expect(err);
            let piece_type = XMLNode::to_piecetype(only_piece.get_attribute("type").expect(err));
            let color = XMLNode::to_color(only_piece.get_attribute("owner").expect(err));
            return (
                shift,
                is_obstacle,
                Some(piece_type),
                Some(color),
                Some(color),
                Vec::new(),
            );
        }
        // more than one piece, first is always bottom piece
        let bottom_piece = pieces_xml.remove(0);
        let piece_type = XMLNode::to_piecetype(bottom_piece.get_attribute("type").expect(err));
        let mut color = XMLNode::to_color(bottom_piece.get_attribute("owner").expect(err));
        let lowest_color = color;
        let mut colors = Vec::new();
        for piece in pieces_xml {
            let inner_color = XMLNode::to_color(piece.get_attribute("owner").expect(err));
            colors.push(inner_color);
            color = inner_color;
        }
        return (
            shift,
            is_obstacle,
            Some(piece_type),
            Some(lowest_color),
            Some(color),
            colors,
        );
    }

    pub fn to_piecetype(info: &str) -> PieceType {
        match info {
            "BEE" => PieceType::BEE,
            "BEETLE" => PieceType::BEETLE,
            "ANT" => PieceType::ANT,
            "GRASSHOPPER" => PieceType::GRASSHOPPER,
            "SPIDER" => PieceType::SPIDER,
            _ => panic!("error while parsing piecetype {}", info),
        }
    }

    pub fn to_color(info: &str) -> Color {
        match info {
            "RED" => Color::RED,
            "BLUE" => Color::BLUE,
            _ => panic!("error while parsing piecetype {}", info),
        }
    }

    pub fn get_name(&self) -> &String {
        return &self.name;
    }

    pub fn get_attributes(&self) -> &HashMap<String, Vec<String>> {
        return &self.attribs;
    }

    pub fn get_attribute(&self, name: &str) -> Option<&String> {
        return self.attribs.get(name).map(|a| &a[0]);
    }

    pub fn get_child_vec(&self, name: &str) -> Vec<&XMLNode> {
        let mut result: Vec<&XMLNode> = Vec::new();

        for child in &self.childs {
            if child.name.as_str() == name {
                result.push(&child);
            }
        }

        return result;
    }

    pub fn get_children(&self) -> &Vec<XMLNode> {
        return &self.childs;
    }

    pub fn get_child(&self, name: &str) -> Option<&XMLNode> {
        for child in &self.childs {
            if child.name.as_str() == name {
                return Some(&child);
            }
        }

        return None;
    }
}

impl Clone for XMLNode {
    fn clone(&self) -> Self {
        return XMLNode {
            name: self.name.clone(),
            data: self.data.clone(),
            attribs: self.attribs.clone(),
            childs: self.childs.clone(),
        };
    }
}
