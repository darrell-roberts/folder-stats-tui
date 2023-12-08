use crate::{
    app::{App, Filter},
    event::{Event, EventHandler},
    folder_stats::collect_folder_stats,
    ui,
};
use anyhow::{Context, Result};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io, panic,
    path::PathBuf,
    sync::{mpsc, Arc},
};

pub type CrosstermTerminal = ratatui::Terminal<ratatui::backend::CrosstermBackend<std::io::Stderr>>;

/// Text user interface.
pub struct Tui {
    terminal: CrosstermTerminal,
    events: EventHandler,
}

impl Tui {
    /// Create a new [`Tui`]
    pub fn new(
        terminal: CrosstermTerminal,
        events: EventHandler,
        root_path: PathBuf,
        depth: usize,
        filters: Arc<Vec<Filter>>,
    ) -> Result<Self> {
        collect_folder_stats(events.sender(), depth, root_path, filters)
            .context("Invalid folder path")?;
        Ok(Self { terminal, events })
    }

    pub fn enter(&mut self) -> Result<()> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(io::stderr(), EnterAlternateScreen, EnableMouseCapture)?;

        let panic_hook = panic::take_hook();
        panic::set_hook(Box::new(move |panic| {
            Self::reset().expect("failed to reset the terminal");
            panic_hook(panic);
        }));

        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        Ok(())
    }

    fn reset() -> Result<()> {
        terminal::disable_raw_mode()?;
        crossterm::execute!(io::stderr(), LeaveAlternateScreen, DisableMouseCapture,)?;
        Ok(())
    }

    pub fn exit(mut self) -> Result<()> {
        Self::reset()?;
        self.terminal.show_cursor()?;
        self.events.shut_down();
        Ok(())
    }

    pub fn draw(&mut self, app: &App) -> Result<()> {
        self.terminal
            .draw(|frame| ui::render(app, frame, self.events.sender()))?;
        Ok(())
    }

    pub fn next_event(&self) -> Result<(Event, mpsc::Sender<Event>)> {
        self.events
            .next()
            .map(|event| (event, self.events.sender()))
    }
}
