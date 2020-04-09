extern crate game_sdk;
extern crate xml;

use self::game_sdk::*;
use self::xml::reader::*;
use super::xml_utils::XMLNode;
use std::io::{prelude::Write, BufReader, BufWriter};
use std::net::TcpStream;

pub struct XMLClient {
    listeners: Vec<Box<dyn ClientListener>>,
    my_color: Option<Color>,
    game_state: Option<GameState>,
    room_id: Option<String>,
}

impl XMLClient {
    pub fn new() -> XMLClient {
        return XMLClient {
            listeners: Vec::new(),
            my_color: None,
            game_state: None,
            room_id: None,
        };
    }

    pub fn add_listener(&mut self, listener: Box<dyn ClientListener>) {
        self.listeners.push(listener);
    }

    /**
     * Runs (and consumes) the client.
     */
    pub fn run(self, target: &String, reservation: &String) {
        println!("Connecting to {}...", target);
        let stream = TcpStream::connect(target).expect("Could not connect to server");

        println!("Connected to {}", &target);
        XMLClient::write_to(&stream, "<protocol>");

        let join_xml: String;
        match reservation.as_str() {
            "" => join_xml = "<join gameType=\"swc_2020_hive\"/>".to_string(),
            _ => join_xml = format!("<joinPrepared reservationCode=\"{}\" />", reservation),
        }

        println!("Sending join message: {}", join_xml);
        XMLClient::write_to(&stream, join_xml.as_str());

        self.handle_stream(&stream);
    }

    fn fire_listeners(&mut self, notifier: &mut dyn FnMut(&mut dyn ClientListener)) {
        let length = self.listeners.len();
        for i in 0..length {
            let boxed: &mut Box<dyn ClientListener> = &mut self.listeners[i];
            let reference = boxed.as_mut();
            notifier(reference);
        }
    }

    fn handle_stream(mut self, stream: &TcpStream) {
        let mut parser = EventReader::new(BufReader::new(stream));

        loop {
            let mut node = XMLNode::read_from(&mut parser);
            /*println!(
                "{} {:?} {:?}",
                node.get_name().as_str(),
                node.get_attributes(),
                node.get_children()
            );*/
            match node.get_name().as_str() {
                "data" => {
                    let invalid = &"".to_string();
                    let data_class = node.get_attribute("class").unwrap_or(invalid).to_string();
                    match data_class.as_str() {
                        "memento" => self.handle_memento_node(&mut node),
                        "welcomeMessage" => self.handle_welcome_message_node(&mut node),
                        "sc.framework.plugins.protocol.MoveRequest" => {
                            let mut default_listener = SimpleClientListener;
                            let move_req_listener: &mut dyn ClientListener;

                            if self.listeners.len() == 0 {
                                move_req_listener = &mut default_listener;
                            } else {
                                move_req_listener = &mut *self.listeners[0];
                            }

                            let game_state = &self
                                .game_state
                                .iter()
                                .clone()
                                .last()
                                .expect("Could not find current game state.");
                            let xml_move =
                                XMLClient::get_move_upon_request(move_req_listener, game_state)
                                    .get_xml(
                                        self.my_color.expect("We don't know of our own color"),
                                    );
                            if let Some(room_id) = &self.room_id {
                                XMLClient::write_to(
                                    stream,
                                    &format!("<room roomId=\"{}\">{}</room>", room_id, xml_move),
                                );
                            } else {
                                println!("error getting room");
                            }
                        }
                        "result" => {
                            /*println!("{:?}", node.get_attributes());
                            println!("{:?}", node.get_children());*/
                            println!("got result");
                            break;
                        }
                        s => {
                            println!("got {}", s.to_string());
                            println!("{:?}", node.get_attributes());
                            println!("{:?}", node.get_children());
                        }
                    }
                }
                "joined" => self.handle_joined_node(&mut node),
                "sc.protocol.responses.CloseConnection" => {
                    println!("Connection closed");
                    break;
                }
                "left" => {
                    println!("left");
                }
                _ => {}
            }
        }
    }

    fn handle_joined_node(&mut self, node: &mut XMLNode) {
        let room = node.as_room();
        self.room_id = Some(room);
    }

    fn handle_memento_node(&mut self, node: &mut XMLNode) {
        let state = node.as_memento();
        self.fire_listeners(&mut |listener| listener.on_update_state(&state));
        self.game_state = Some(state);
    }

    fn handle_welcome_message_node(&mut self, node: &mut XMLNode) {
        let color = node.as_welcome_message();
        self.fire_listeners(&mut |listener| listener.on_welcome_message(color));
        self.my_color = Some(color);
    }

    fn get_move_upon_request(
        move_req_listener: &mut dyn ClientListener,
        game_state: &GameState,
    ) -> Action {
        return move_req_listener.on_move_request(game_state);
    }

    fn write_to(stream: &TcpStream, data: &str) {
        let _ = BufWriter::new(stream).write(data.as_bytes());
    }
}

pub struct SimpleClientListener;

impl ClientListener for SimpleClientListener {
    fn on_move_request(&mut self, state: &GameState) -> Action {
        println!("{}", state);
        let mut al = ActionList::default();
        gamerules::calculate_legal_moves(state, &mut al);
        if al.size > 0 {
            println!("sending {}", al[0].get_xml(Color::RED));
            return al[0];
        }
        panic!("Did not find move");
    }
}
