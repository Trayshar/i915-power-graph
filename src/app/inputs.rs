use std::{
    sync::mpsc::{channel, Receiver, RecvTimeoutError},
    time::Duration, thread,
};

use crossterm::event::{self, KeyEvent};

pub enum InputEvent {
    Key(KeyEvent),
    None,
}

pub struct InputThread {
    rx: Receiver<InputEvent>,
    tick_rate: Duration
}

impl InputThread {
    pub fn new(tick_rate: Duration) -> InputThread {
        let (tx, rx) = channel();

        thread::spawn(move || {
            loop {
                // Poll for tick rate duration, if no event, sent tick event
                // Unwraping is not a problem: If these operations fail, it means the main thread died
                // In this case, this thread will panic and kill itself as well
                if event::poll(tick_rate).unwrap() {
                    if let event::Event::Key(key) = event::read().unwrap() {
                        tx.send(InputEvent::Key(key)).unwrap();
                    }
                }
                tx.send(InputEvent::None).unwrap();
            }
        });

        InputThread { rx, tick_rate }
    }

    /// Attempts to read an event.
    /// This function blocks the current thread.
    pub fn next(&self) -> Result<InputEvent, RecvTimeoutError> {
        self.rx.recv_timeout(self.tick_rate * 2)
    }
}