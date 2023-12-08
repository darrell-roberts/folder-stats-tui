use std::sync::Arc;

use anyhow::Result;
use app::App;
use args::Args;
use clap::Parser;
use event::EventHandler;
use flexi_logger::{FileSpec, Logger};
use log::error;
use ratatui::{backend::CrosstermBackend, Terminal};
use tui::Tui;
use update::handle_event;

pub mod app;
pub mod args;
pub mod event;
// pub mod folder_stats;
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

    let root_path = args.root_path.canonicalize()?;
    let depth = args.depth;
    let filters = Arc::new(args.filters());

    let backend = CrosstermBackend::new(std::io::stderr());
    let terminal = Terminal::new(backend)?;
    let mut tui = Tui::new(
        terminal,
        EventHandler::new(250),
        root_path.clone(),
        depth,
        filters.clone(),
    )?;

    let mut app = App::new(root_path, filters.clone());

    tui.enter()?;

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
