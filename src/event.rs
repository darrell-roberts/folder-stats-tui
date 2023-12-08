use anyhow::Result;
use crossterm::event::{self, KeyEvent, MouseEvent};
use log::error;
use std::{
    ops::Not,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread::{self},
    time::{Duration, Instant},
};

static SHUTTING_DOWN: AtomicBool = AtomicBool::new(false);

#[derive(Clone, Debug)]
pub enum Event {
    Tick,
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Progress(String),
    ScanComplete,
    ContentFrameSize(u16),
    FolderEvent(Vec<(String, u64)>),
    WorkerStart,
    WorkerEnd,
}

#[derive(Debug)]
pub struct EventHandler {
    sender: mpsc::Sender<Event>,
    receiver: mpsc::Receiver<Event>,
    handler: thread::JoinHandle<()>,
}

impl EventHandler {
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (sender, receiver) = mpsc::channel();
        let handler = {
            let sender = sender.clone();
            thread::spawn(move || {
                let mut last_tick = Instant::now();
                while SHUTTING_DOWN.load(Ordering::Acquire).not() {
                    let timeout = tick_rate
                        .checked_sub(last_tick.elapsed())
                        .unwrap_or(tick_rate);

                    let poll = event::poll(timeout).unwrap_or_else(|err| {
                        error!("Failed to poll terminal event: {err}");
                        false
                    });
                    if poll {
                        match event::read() {
                            Ok(e) => {
                                let r = match e {
                                    event::Event::Key(e) => {
                                        if e.kind == event::KeyEventKind::Press {
                                            sender.send(Event::Key(e))
                                        } else {
                                            Ok(())
                                        }
                                    }
                                    event::Event::Mouse(e) => sender.send(Event::Mouse(e)),
                                    event::Event::Resize(w, h) => sender.send(Event::Resize(w, h)),
                                    _ => Ok(()),
                                };

                                if let Err(err) = r {
                                    error!("Failed to send event: {err}");
                                }
                            }
                            Err(err) => error!("Failed to read terminal event: {err}"),
                        }
                    }

                    if last_tick.elapsed() >= tick_rate {
                        if let Err(err) = sender.send(Event::Tick) {
                            error!("Failed to send Tick: {err}");
                        }
                        last_tick = Instant::now();
                    }
                }
            })
        };
        Self {
            sender,
            receiver,
            handler,
        }
    }

    pub fn sender(&self) -> mpsc::Sender<Event> {
        self.sender.clone()
    }

    pub fn next(&self) -> Result<Event> {
        Ok(self.receiver.recv()?)
    }

    pub fn shut_down(self) {
        SHUTTING_DOWN.store(true, Ordering::Release);
        if self.handler.join().is_err() {
            error!("Failed to wait on event thread");
        }
    }
}
