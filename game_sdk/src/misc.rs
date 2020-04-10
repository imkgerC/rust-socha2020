use crate::GameState;
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
}
impl IntoIterator for FenReader {
    type Item = (GameState, Vec<u64>);
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
    type Item = (GameState, Vec<u64>);

    fn next(&mut self) -> Option<Self::Item> {
        let line = self.lines.pop();
        if line.is_some() {
            let line = line.unwrap();
            let s: Vec<&str> = line.split("/").collect();
            let fen = s[0];
            let perfts: Vec<u64> = s[1]
                .split(" ")
                .into_iter()
                .map(|s| s.replace("\r", "").parse::<u64>().unwrap())
                .collect();
            let state = GameState::from_fen(fen.to_owned());
            Some((state, perfts))
        } else {
            None
        }
    }
}
