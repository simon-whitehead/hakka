
use std;
use std::io::{self, BufRead, Write};
use std::sync::mpsc::Sender;
use std::thread;

pub fn handle(sender: Sender<String>) {
    thread::spawn(move || {
        loop {
            print!("hakka> ");
            std::io::stdout().flush();

            let mut line = String::new();
            let stdin = io::stdin();
            stdin.lock().read_line(&mut line).expect("Could not read line");
            sender.send(line).unwrap();
        }
    });
}