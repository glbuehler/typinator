use std::{sync::mpsc, thread};

use crossterm::event;

pub fn spawn_input_thread() -> mpsc::Receiver<event::Event> {
    let (send, recv) = mpsc::channel();

    thread::spawn(move || {
        loop {
            match event::read() {
                Ok(e) => {
                    if let Err(_) = send.send(e) {
                        break;
                    }
                }
                Err(e) => {
                    println!("{e}");
                }
            }
        }
    });

    recv
}
