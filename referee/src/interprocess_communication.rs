use crate::logging::Log;
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{ChildStderr, ChildStdin, ChildStdout};
use std::time::Duration;

pub fn print_command(stdin: &mut ChildStdin, command: String) {
    stdin
        .write_all(command.as_bytes())
        .expect("Could not write to StdIn of child")
}
pub fn block_on_output<F: Fn(String) -> bool>(
    stdout: ChildStdout,
    functor: F,
    stderr: &mut ChildStderr,
    log: &mut Log,
) -> (String, ChildStdout) {
    let mut buf_string = String::new();
    let mut bufreader = BufReader::new(stdout);
    loop {
        let mut new_line = String::new();
        bufreader
            .read_line(&mut new_line)
            .expect("Could not read line");
        buf_string.push_str(&new_line);
        if functor(new_line.clone()) {
            break;
        } else if new_line.len() == 0 {
            log.log(&buf_string, false);
            log_stderr(stderr, log);
            panic!("");
        }
    }
    (buf_string, bufreader.into_inner())
}
pub fn log_stderr(stderr: &mut ChildStderr, log: &mut Log) {
    log.log("StdERR:\n", true);
    let mut buffer = Vec::new();
    stderr
        .read_to_end(&mut buffer)
        .expect("Could not read from stderr");
    log.log(
        std::str::from_utf8(&buffer).expect("Could not convert to utf8"),
        true,
    );
}
