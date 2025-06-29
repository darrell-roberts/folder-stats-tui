use crate::{
    app::{App, FolderStat, SortBy},
    event::Event,
    walker::collect_stats,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use std::{cmp::Reverse, sync::mpsc};

fn handle_key_event(app: &mut App, key_event: KeyEvent, sender: mpsc::Sender<Event>) {
    match key_event.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            if app.show_help {
                app.show_help = false;
            } else {
                app.quit();
            }
        }
        KeyCode::Char('c') | KeyCode::Char('C') if key_event.modifiers == KeyModifiers::CONTROL => {
            app.quit()
        }
        KeyCode::Char('s') => handle_sort(app, SortBy::FileSize),
        KeyCode::Char('c') => handle_sort(app, SortBy::FileCount),
        KeyCode::Up | KeyCode::Char('k') => app.scroll_up(1),
        KeyCode::Down | KeyCode::Char('j') => app.scroll_down(1),
        KeyCode::PageUp => app.scroll_up(app.compute_scroll_page()),
        KeyCode::PageDown => app.scroll_down(app.compute_scroll_page()),
        KeyCode::Home => app.scroll_state = 0,
        KeyCode::End => app.scroll_state = app.max_scroll,
        KeyCode::Char('1') => handle_depth_change(app, 1, sender),
        KeyCode::Char('2') => handle_depth_change(app, 2, sender),
        KeyCode::Char('3') => handle_depth_change(app, 3, sender),
        KeyCode::Char('4') => handle_depth_change(app, 4, sender),
        KeyCode::Char('5') => handle_depth_change(app, 5, sender),
        KeyCode::Char('6') => handle_depth_change(app, 6, sender),
        KeyCode::Char('7') => handle_depth_change(app, 7, sender),
        KeyCode::Char('8') => handle_depth_change(app, 8, sender),
        KeyCode::Char('?') => app.show_help = !app.show_help,
        KeyCode::Char('u') | KeyCode::Char('b') if key_event.modifiers == KeyModifiers::CONTROL => {
            app.scroll_up(app.compute_scroll_page());
        }
        KeyCode::Char('d') | KeyCode::Char('f') if key_event.modifiers == KeyModifiers::CONTROL => {
            app.scroll_down(app.compute_scroll_page());
        }
        KeyCode::Char('i') => toggle_ignores(app, sender),
        KeyCode::Char('h') => toggle_hidden(app, sender),

        _ => (),
    }
}

fn handle_depth_change(app: &mut App, depth: u8, sender: mpsc::Sender<Event>) {
    if app.scanning {
        return;
    }
    app.scanning = true;
    app.config.depth = depth;

    collect_stats(sender, app.config);
}

fn toggle_ignores(app: &mut App, sender: mpsc::Sender<Event>) {
    if app.scanning {
        return;
    }
    app.config.no_ignores = !app.config.no_ignores;
    app.scanning = true;

    collect_stats(sender, app.config);
}

fn toggle_hidden(app: &mut App, sender: mpsc::Sender<Event>) {
    if app.scanning {
        return;
    }
    app.scanning = true;
    app.config.show_hidden = !app.config.show_hidden;
    collect_stats(sender, app.config);
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
    app.scan_result.sort_unstable_by_key(|(_, stats)| {
        Reverse(match sort_by {
            SortBy::FileSize => stats.size as usize,
            SortBy::FileCount => stats.files,
        })
    });
    app.scroll_state = 0;
}

/// Main event handler.
pub fn handle_event(app: &mut App, event: Event, sender: mpsc::Sender<Event>) {
    match event {
        Event::Key(key_event) => handle_key_event(app, key_event, sender),
        Event::Progress(folder) => app.update_progress(folder),
        Event::ScanComplete(elapsed) => {
            let mut sorted_result = std::mem::take(&mut app.folder_events)
                .into_iter()
                .collect::<Vec<_>>();
            sorted_result.sort_unstable_by_key(|(_, stat)| Reverse(stat.size));
            app.scan_result = sorted_result;
            app.compute_max_scroll();
            app.scanning = false;
            app.scan_time = elapsed;
        }
        Event::Mouse(mouse_event) => handle_mouse_event(app, mouse_event),
        Event::Resize(_, h) => {
            app.content_height = h.checked_sub(5).unwrap_or(h);
            app.compute_max_scroll();
        }
        Event::ContentFrameSize(h) => {
            app.content_height = h.checked_sub(2).unwrap_or(h);
            app.compute_max_scroll()
        }
        Event::FolderEvent(events) => {
            for (folder_name, stats) in events {
                app.folder_events
                    .entry(folder_name)
                    .and_modify(|fs: &mut FolderStat| {
                        fs.size += stats.size;
                        fs.files += stats.files;
                    })
                    .or_insert(FolderStat {
                        size: stats.size,
                        files: stats.files,
                    });
            }
        }
        _ => (),
    }
}
