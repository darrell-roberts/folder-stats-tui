use crate::app::FolderStat;
use anyhow::Result;
use crossterm::event::{self, KeyEvent, MouseEvent};
use log::error;
use std::{
    collections::HashMap,
    ops::Not,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread::{self},
    time::{Duration, Instant},
};

static SHUTTING_DOWN: AtomicBool = AtomicBool::new(false);

/// Events emitted by Walker and Crossterm.
#[derive(Clone, Debug)]
pub enum Event {
    Tick,
    /// Crossterm key event.
    Key(KeyEvent),
    /// Crossterm mouse event.
    Mouse(MouseEvent),
    /// Crossterm resize event.
    Resize(u16, u16),
    /// Walker scan progress. Emits folder being scanned.
    Progress(String),
    /// Walker scan completed.
    ScanComplete,
    /// Initial rendered content frame size.
    ContentFrameSize(u16),
    /// Walker parallel worker folder collection.
    FolderEvent(HashMap<String, FolderStat>),
}

/// Application event handler.
#[derive(Debug)]
pub struct EventHandler {
    sender: mpsc::Sender<Event>,
    receiver: mpsc::Receiver<Event>,
    handler: thread::JoinHandle<()>,
}

impl EventHandler {
    /// Create an application event handler with the given tick rate.
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
                            Ok(event) => {
                                let status = match event {
                                    event::Event::Key(e)
                                        if e.kind == event::KeyEventKind::Press =>
                                    {
                                        sender.send(Event::Key(e))
                                    }
                                    event::Event::Mouse(e) => sender.send(Event::Mouse(e)),
                                    event::Event::Resize(w, h) => sender.send(Event::Resize(w, h)),
                                    _ => Ok(()),
                                };

                                if let Err(err) = status {
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

    /// Create a sender to emit to this event handler.
    pub fn sender(&self) -> mpsc::Sender<Event> {
        self.sender.clone()
    }

    /// Receive the next emitted event.
    pub fn next(&self) -> Result<Event> {
        Ok(self.receiver.recv()?)
    }

    /// Shut down the event handler.
    pub fn shut_down(self) {
        SHUTTING_DOWN.store(true, Ordering::Release);
        if self.handler.join().is_err() {
            error!("Failed to wait on event thread");
        }
    }
}
