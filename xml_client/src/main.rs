extern crate player;
mod xml_client;
mod xml_utils;

use argparse::{ArgumentParser, Store};
use player::mcts::MCTS as Player;
// use player::search::Searcher as Player;
use xml_client::XMLClient;

fn main() {
    let mut host = "localhost".to_string();
    let mut port = "13050".to_string();
    let mut reservation = "".to_string();
    {
        let mut ap = ArgumentParser::new();
        ap.refer(&mut host)
            .add_option(&["-h", "--host"], Store, "Host to run game on");
        ap.refer(&mut port)
            .add_option(&["-p", "--port"], Store, "Port to run game on");
        ap.refer(&mut reservation).add_option(
            &["-r", "--reservation"],
            Store,
            "Reservation to join",
        );
        ap.parse_args_or_exit();
    }
    println!(
        "Parameters got are: {}:{} w/ reservation {}",
        host, port, reservation
    );
    let mut client = XMLClient::new();
    // Insert custom client listener here:
    client.add_listener(Box::new(Player::new()));

    client.run(&(host + ":" + port.as_str()), &reservation);
}
