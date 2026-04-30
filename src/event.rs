use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use crossterm::event::Event as CtEvent;

pub enum AppEvent {
    Input(CtEvent),
    Tick,
}

pub struct EventChannel {
    pub rx: Receiver<AppEvent>,
    _tx: Sender<AppEvent>,
}

impl EventChannel {
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::channel();

        let tx_input = tx.clone();
        thread::spawn(move || loop {
            if crossterm::event::poll(Duration::from_millis(50)).unwrap_or(false) {
                if let Ok(event) = crossterm::event::read() {
                    if tx_input.send(AppEvent::Input(event)).is_err() {
                        return;
                    }
                }
            }
        });

        let tx_tick = tx.clone();
        thread::spawn(move || loop {
            thread::sleep(tick_rate);
            if tx_tick.send(AppEvent::Tick).is_err() {
                return;
            }
        });

        Self { rx, _tx: tx }
    }
}
