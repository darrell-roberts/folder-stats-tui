use anyhow::Result;
use app::{App, Config};
use args::Args;
use clap::Parser;
use event::EventHandler;
use flexi_logger::{FileSpec, Logger};
use log::error;
use ratatui::{backend::CrosstermBackend, Terminal};
use tui::Tui;
use update::handle_event;
use walker::collect_stats;

pub mod app;
pub mod args;
pub mod event;
pub mod tui;
pub mod ui;
pub mod update;
pub mod walker;

fn main() -> Result<()> {
    Logger::try_with_str("info")?
        .log_to_file(FileSpec::default().basename("f-stats").suffix("log"))
        .print_message()
        .start()?;

    let args = Args::parse();

    let config = Config::try_from(args)?;

    let backend = CrosstermBackend::new(std::io::stderr());
    let terminal = Terminal::new(backend)?;
    let event_handler = EventHandler::new(250);
    let sender = event_handler.sender();
    let mut tui = Tui::new(terminal, event_handler)?;

    let mut app = App::new(config);

    tui.enter()?;

    // Draw the initial screen
    if let Err(err) = tui.draw(&app) {
        error!("Failed to draw tui: {err}");
    }

    collect_stats(sender, config);

    // Main event loop.
    while !app.should_quit {
        if let Err(err) = tui.draw(&app) {
            error!("Failed to draw tui: {err}");
            break;
        }

        match tui.next_event() {
            Ok((event, sender)) => {
                handle_event(&mut app, event, sender);
            }
            Err(err) => {
                error!("Failed to read next event: {err}");
            }
        }
    }

    tui.exit()?;

    Ok(())
}
