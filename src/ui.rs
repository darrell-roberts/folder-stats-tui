use crate::{
    app::{App, SortBy},
    event::Event,
};
use bytesize::ByteSize;
use log::error;
use ratatui::{
    layout::{Constraint, Direction, Layout, Margin},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Bar, BarChart, BarGroup, Block, BorderType, Borders, Paragraph, Scrollbar,
        ScrollbarOrientation, ScrollbarState,
    },
    Frame,
};
use std::sync::mpsc;

/// Render the Tui based on the [`App`] current state.
pub fn render(app: &App, frame: &mut Frame, sender: mpsc::Sender<Event>) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(10)])
        .split(frame.size());

    // Only emit this once on the first render when we have a result.
    if app.content_height == 0 && !app.scanning {
        if let Err(err) = sender.send(Event::ContentFrameSize(rows[1].height)) {
            error!("Failed to emit content frame height: {err}");
        }
    }

    // The root entry has the recursive count for both file size and
    // total files.
    let (root_folder, total_size, total_files) = app
        .scan_result
        .iter()
        .last()
        .map(|(f, v)| (f.as_str(), v.size, v.files))
        .unwrap_or_default();

    frame.render_widget(
        Paragraph::new(if app.scanning {
            vec![Line::from(vec![
                Span::raw("scanning folder: "),
                Span::styled(&app.folder_name, Style::default().red()),
            ])]
        } else {
            let blue = Style::default().light_blue();
            let red = Style::default().red();
            vec![
                Line::from(vec![
                    Span::styled("Scan results for: ", blue),
                    Span::styled(root_folder, red),
                ]),
                Line::from(vec![
                    Span::styled("Total Size: ", blue),
                    Span::styled(format!("{} ", ByteSize(total_size)), red),
                    Span::styled("Total folders: ", blue),
                    Span::styled(
                        format!(
                            "{} ",
                            app.scan_result.len().checked_sub(1).unwrap_or_default()
                        ),
                        red,
                    ),
                    Span::styled("Total Files: ", blue),
                    Span::styled(format!("{total_files} "), red),
                ]),
                Line::from(
                    vec![
                        Span::styled("Folder depth: ", blue),
                        Span::styled(format!("{} ", &app.depth), red),
                    ], // format!(
                       //     "Folder depth: {} Max scroll: {} Scroll: {} content height: {}",
                       //     &app.depth, &app.max_scroll, &app.scroll_state, rows[1].height
                       // )
                       // .cyan(),
                ),
            ]
        })
        .block(
            Block::default()
                .title(if app.scanning {
                    "Scan progress"
                } else {
                    "Folder stats"
                })
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray)),
            // .style(Style::default().bg(Color::White)),
        ),
        rows[0],
    );

    render_content(app, frame, rows[1], total_size, total_files);
}

/// Render the content section.
fn render_content(
    app: &App,
    frame: &mut Frame<'_>,
    row: ratatui::prelude::Rect,
    total_size: u64,
    total_files: usize,
) {
    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("↑"))
        .end_symbol(Some("↓"));

    let bar_groups = app
        .scan_result
        .iter()
        .rev()
        .enumerate()
        .skip(app.scroll_state + 1)
        .map(|(index, (name, stats))| {
            let bar_file_size = (stats.size as f32 / total_size as f32) * 100.;
            let bar_file_num = (stats.files as f32 / total_files as f32) * 100.;
            let bars = &[
                Bar::default()
                    .value(bar_file_size as u64)
                    .style(Style::new().red())
                    .value_style(Style::new().black().on_red())
                    .text_value(format!("{}", ByteSize(stats.size))),
                Bar::default()
                    .value(bar_file_num as u64)
                    .style(Style::new().magenta())
                    .value_style(Style::new().black().on_magenta())
                    .text_value(format!("{} files", stats.files)),
            ];
            let name = format!("{index}. {name}");
            BarGroup::default().label(name.into()).bars(bars)
        });

    let mut scrollbar_state = ScrollbarState::new(app.max_scroll)
        .position(app.scroll_state)
        .viewport_content_length(App::ITEM_HEIGHT as usize);

    let mut chart = BarChart::default()
        .direction(Direction::Horizontal)
        .block(
            Block::default()
                .title(if matches!(app.sort, SortBy::FileSize) {
                    "Largest by Size"
                } else {
                    "Largest by File Count"
                })
                .border_style(Style::default().fg(Color::DarkGray))
                // .style(Style::default().bg(Color::White))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded),
        )
        .bar_width(1)
        .bar_gap(0)
        .group_gap(2)
        .label_style(Style::new().blue().bold())
        .max(100);

    for g in bar_groups {
        chart = chart.data(g);
    }

    frame.render_widget(chart, row);
    let area = row;
    frame.render_stateful_widget(
        scrollbar,
        area.inner(&Margin {
            vertical: 1,
            horizontal: 0,
        }),
        &mut scrollbar_state,
    );
}
