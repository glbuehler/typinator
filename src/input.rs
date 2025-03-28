use std::{
    sync::{Arc, atomic, mpsc},
    thread,
};

use crossterm::event;

pub fn spawn_input_thread() -> (mpsc::Receiver<event::Event>, Arc<atomic::AtomicBool>) {
    let (send, recv) = mpsc::channel();
    let ret = Arc::new(atomic::AtomicBool::new(true));
    let run = ret.clone();

    thread::spawn(move || {
        while run.load(atomic::Ordering::Relaxed) {
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

    (recv, ret)
}
