use std::fs::{File, OpenOptions};
use std::io::Write;

pub struct Log(File);
impl Log {
    pub fn init(path: &str, append: bool) -> Log {
        if !append {
            let _ = std::fs::remove_file(path);
        }
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(append)
            .open(path)
            .expect("please create a dir 'referee_logs'");
        Log(file)
    }
    pub fn log(&mut self, msg: &str, also_stdout: bool) {
        self.0
            .write(msg.as_bytes())
            .expect("Could not write to log");
        if also_stdout {
            println!("{}", msg);
        }
    }
}
