# Rust socha 2020
This is a client in rust for the 2020 edition of [Software-Challenge Germany](https://software-challenge.de/). It includes basic functionality for communicating through the official XML-based protocol and implements all rules of the game as implemented by [CAU](https://github.com/CAU-Kiel-Tech-Inf/socha). 

Currently, there are two known problems with the rules/their standard implementation. If you can not place the bee when you need to, the rules do not specify what should happen. The GUI is inconsistent in that it requires SkipMoves most of the time but sometimes just bugs out. If you implement any kind of player with this framework, you need to handle this problem somehow. One way would be to allow a SkipMove then or just let the player, that is not able to place a bee lose. Another problem with the rules is one of consistency with the original game of hive. In the standard implementation beetles are allowed to move to fields, that are not accessible to bees, which is not allowed in the original game of hive. This implementation mirrors the behaviour of the CAU-implementation and allows those moves.

## Prerequisites
To build/run you need a rather current version of cargo. The easiest way to install and manage cargo is through [rustup](https://rustup.rs/). This framework was developed and tested with rustc 1.40.0.

To test your implementation you need tools provided by the CAU, such as [GUI](https://github.com/CAU-Kiel-Tech-Inf/socha-gui/releases) and [test server](https://github.com/CAU-Kiel-Tech-Inf/socha/releases). You can also use our own referee with better readability (elo stats, LOS, ...) and better stability, as the official test server can crash sometimes. Our referee only accepts clients implementing our own stdin/stdout framework

## Usage
To build a client for use with official tools you need to build `xml_client`. It is strongly advised against testing your player in debug mode, as the `game_sdk` then checks integrity on every move. This degrades performance by multiple orders of magnitude. To build use ```cargo build -p xml_client --release```, you will then find an executable under `./target/release/xml_client.exe` that can be used in the GUI or with the test server. To directly run the executable you can invoke ```cargo run -p xml_client --release```. To build for the online system you need to specify a different toolchain, you can use either `x86_64-unknown-linux-gnu` or if there are problems with the linked version of libc, then `x86_64-unknown-linux-musl`.

For a quick performance demonstration you can run the `demo` crate. ```cargo run -p demo --release```.
