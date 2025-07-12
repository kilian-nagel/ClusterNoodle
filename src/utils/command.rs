use std::io;
use std::process::{Command, Output};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

pub fn run_with_timeout(mut command: Command, timeout: Duration) -> io::Result<Option<Output>> {
    let (sender, receiver) = mpsc::channel();

    thread::spawn(move || {
        let result = command.output(); // This is blocking
        let _ = sender.send(result);
    });

    match receiver.recv_timeout(timeout) {
        Ok(result) => result.map(Some),                   // Command finished
        Err(mpsc::RecvTimeoutError::Timeout) => Ok(None), // Timeout occurred
        Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)), // Other errors
    }
}
