use crate::gamerules::{get_result, is_game_finished};
use crate::{Color, GameState};
use std::fs::{File, OpenOptions};
use std::io::{BufReader, Read};

pub struct FenReader(File);
impl FenReader {
    pub fn from_path(path: &str) -> Self {
        let file = OpenOptions::new()
            .read(true)
            .open(path)
            .expect("Invalid path");
        FenReader(file)
    }
    pub fn into_game_reader(self) -> GameReader {
        GameReader(self.into_iter())
    }
}
impl IntoIterator for FenReader {
    type Item = (GameState, String);
    type IntoIter = FenReaderState;

    fn into_iter(self) -> Self::IntoIter {
        let mut contents = String::new();
        let mut buf_reader = BufReader::new(self.0);
        buf_reader
            .read_to_string(&mut contents)
            .expect("Could not read from file");
        let lines: Vec<&str> = contents.split("\n").collect();
        let lines: Vec<String> = lines.iter().map(|&s| s.to_owned()).collect();
        FenReaderState { lines }
    }
}
pub struct FenReaderState {
    lines: Vec<String>,
}
impl Iterator for FenReaderState {
    type Item = (GameState, String);

    fn next(&mut self) -> Option<Self::Item> {
        if !self.lines.is_empty() {
            let line = self.lines.remove(0);
            let s: Vec<&str> = line.split("//").collect();
            let fen = s[0];
            if s[0].is_empty() {
                self.next()
            } else {
                let state = GameState::from_fen(fen.to_owned());
                Some((state, s[1].to_owned()))
            }
        } else {
            None
        }
    }
}
pub struct GameReader(FenReaderState);
impl Iterator for GameReader {
    type Item = (Vec<GameState>, Option<Color>);

    fn next(&mut self) -> Option<Self::Item> {
        let mut states = Vec::with_capacity(60);
        let mut result = None;
        while let Some((state, desc)) = self.0.next() {
            if is_game_finished(&state) {
                result = get_result(&state);
                break;
            }
            let search_res = desc
                .replace("Some(", "")
                .replace(")", "")
                .parse::<i16>()
                .expect(desc.as_str());
            states.push(state);
            /*if search_res.abs() < 29900 {
                states.push(state);
            }*/
        }
        if states.is_empty() {
            None
        } else {
            Some((states, result))
        }
    }
}
