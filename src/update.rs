use crate::{
    app::{App, SortBy},
    event::Event,
    folder_stats::collect_folder_stats,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use log::error;
use std::sync::mpsc;

fn handle_key_event(app: &mut App, key_event: KeyEvent, sender: mpsc::Sender<Event>) {
    match key_event.code {
        KeyCode::Esc | KeyCode::Char('q') => app.quit(),
        KeyCode::Char('c') | KeyCode::Char('C') if key_event.modifiers == KeyModifiers::CONTROL => {
            app.quit()
        }
        KeyCode::Char('s') => handle_sort(app, SortBy::FileSize),
        KeyCode::Char('c') => handle_sort(app, SortBy::FileCount),
        KeyCode::Up => app.scroll_up(1),
        KeyCode::Down => app.scroll_down(1),
        KeyCode::PageUp => app.scroll_up(app.compute_scroll_page()),
        KeyCode::PageDown => app.scroll_down(app.compute_scroll_page()),
        KeyCode::Home => app.scroll_state = 0,
        KeyCode::End => app.scroll_state = app.max_scroll,
        KeyCode::Char('1') => handle_depth_change(app, 1, sender),
        KeyCode::Char('2') => handle_depth_change(app, 2, sender),
        KeyCode::Char('3') => handle_depth_change(app, 3, sender),
        KeyCode::Char('4') => handle_depth_change(app, 4, sender),
        KeyCode::Char('5') => handle_depth_change(app, 5, sender),
        _ => (),
    }
}

fn handle_depth_change(app: &mut App, depth: usize, sender: mpsc::Sender<Event>) {
    if app.scanning {
        return;
    }
    app.scanning = true;
    app.depth = depth;
    if let Err(err) = collect_folder_stats(sender, depth) {
        error!("Failed to change folder depth: {err}");
    }
}

fn handle_mouse_event(app: &mut App, mouse_event: MouseEvent) {
    match mouse_event.kind {
        crossterm::event::MouseEventKind::ScrollDown => app.scroll_down(1),
        crossterm::event::MouseEventKind::ScrollUp => app.scroll_up(1),
        _ => (),
    }
}

fn handle_sort(app: &mut App, sort_by: SortBy) {
    app.sort = sort_by;
    match sort_by {
        SortBy::FileSize => {
            app.scan_result
                .sort_unstable_by_key(|(_, stats)| stats.size);
            app.scroll_state = 0;
        }
        SortBy::FileCount => {
            app.scan_result
                .sort_unstable_by_key(|(_, stats)| stats.files);
            app.scroll_state = 0;
        }
    }
}

pub fn handle_event(app: &mut App, event: Event, sender: mpsc::Sender<Event>) {
    match event {
        Event::Key(key_event) => handle_key_event(app, key_event, sender),
        Event::Progress(folder) => app.update_progress(folder),
        Event::ScanComplete(result) => {
            app.scanning = false;
            app.update_scan_result(result);
        }
        Event::Mouse(mouse_event) => handle_mouse_event(app, mouse_event),
        Event::Resize(_, h) => {
            // debug!("Resized window height {h}");
            app.content_height = h.checked_sub(5).unwrap_or(h);
            app.compute_max_scroll();
        }
        Event::ContentFrameSize(h) => {
            // debug!("Initial content height {h}");
            app.content_height = h.checked_sub(2).unwrap_or(h);
            app.compute_max_scroll()
        }
        _ => (),
    }
}
