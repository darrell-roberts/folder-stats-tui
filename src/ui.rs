use crate::{
    app::{App, Filter, SortBy},
    event::Event,
};
use bytesize::ByteSize;
use log::error;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin},
    prelude::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Bar, BarChart, BarGroup, Block, BorderType, Borders, Cell, Clear, Paragraph, Row,
        Scrollbar, ScrollbarOrientation, ScrollbarState, Table,
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
    let (total_size, total_files) = app
        .scan_result
        .iter()
        .last()
        .map(|(_, v)| (v.size, v.files))
        .unwrap_or_default();

    render_header(app, frame, rows[0], total_files, total_size);
    render_content(app, frame, rows[1], total_size, total_files);

    if app.show_help {
        render_help(frame);
    }
}

fn render_help(frame: &mut Frame) {
    let blue = Style::default().light_blue();
    let red = Style::default().red();
    let block = Block::default()
        .title("Help")
        .borders(Borders::ALL)
        .title_alignment(Alignment::Center);
    let table = Table::new(vec![
        Row::new(vec![
            Cell::from(Line::styled("1 - 8", blue)),
            Cell::from(Line::styled("Change folder depth", red)),
        ]),
        Row::new(vec![
            Cell::from(Line::styled("c", blue)),
            Cell::from(Line::styled("Sort by file count", red)),
        ]),
        Row::new(vec![
            Cell::from(Line::styled("s", blue)),
            Cell::from(Line::styled("Sort by file size", red)),
        ]),
        Row::new(vec![
            Cell::from(Line::styled("k - up", blue)),
            Cell::from(Line::styled("Up", red)),
        ]),
        Row::new(vec![
            Cell::from(Line::styled("j - down", blue)),
            Cell::from(Line::styled("Down", red)),
        ]),
        Row::new(vec![
            Cell::from(Line::styled("pgup", blue)),
            Cell::from(Line::styled("Page Up", red)),
        ]),
        Row::new(vec![
            Cell::from(Line::styled("pgdn", blue)),
            Cell::from(Line::styled("Page Down", red)),
        ]),
        Row::new(vec![
            Cell::from(Line::styled("q - ESC", blue)),
            Cell::from(Line::styled("quit", red)),
        ]),
    ])
    .block(block)
    .header(Row::new(vec!["Key", "Usage"]).bottom_margin(1))
    .widths(&[Constraint::Length(8), Constraint::Percentage(60)])
    .column_spacing(1);

    let area = centered_rect(19, 28, frame.size());
    frame.render_widget(Clear, area);
    frame.render_widget(table, area);
}

fn render_header(
    app: &App,
    frame: &mut Frame,
    row: ratatui::prelude::Rect,
    total_files: usize,
    total_size: u64,
) {
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
                    Span::styled(app.root_folder(), red),
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
                Line::from(vec![
                    Span::styled("Folder depth: ", blue),
                    Span::styled(format!("{} ", &app.depth), red),
                    Span::styled("Filter: ", blue),
                    Span::styled(
                        format!(
                            "{} ",
                            app.config
                                .filters
                                .iter()
                                .filter_map(|f| {
                                    match f {
                                        Filter::FileName(s) => Some(s),
                                        Filter::Extension(_) => None,
                                    }
                                })
                                .fold(String::new(), |mut filter, f| {
                                    filter.push_str(f);
                                    filter
                                })
                        ),
                        red,
                    ),
                    Span::styled("Extension Filter: ", blue),
                    Span::styled(
                        format!(
                            "{} ",
                            app.config
                                .filters
                                .iter()
                                .filter_map(|f| {
                                    match f {
                                        Filter::FileName(_) => None,
                                        Filter::Extension(s) => Some(s),
                                    }
                                })
                                .fold(String::new(), |mut filter, f| {
                                    filter.push('.');
                                    filter.push_str(f);
                                    filter
                                })
                        ),
                        red,
                    ),
                    Span::styled("ignores: ", blue),
                    Span::styled(format!("{}", !app.config.no_ignores), red),
                ]),
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
        ),
        row,
    );
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

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
