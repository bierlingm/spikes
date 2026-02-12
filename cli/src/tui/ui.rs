use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState, Wrap},
    Frame,
};

use super::app::{App, InputMode};
use crate::spike::Rating;

// Brand colors
const RED: Color = Color::Rgb(231, 76, 60);      // #e74c3c
const GREEN: Color = Color::Rgb(34, 197, 94);    // #22c55e
const BLUE: Color = Color::Rgb(59, 130, 246);    // #3b82f6
const YELLOW: Color = Color::Rgb(234, 179, 8);   // #eab308
const TEXT_MUTED: Color = Color::Rgb(161, 161, 170); // #a1a1aa
const TEXT_DIM: Color = Color::Rgb(82, 82, 91);  // #52525b

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(3), // Filter bar
            Constraint::Min(5),    // Table
            Constraint::Length(if app.show_detail { 10 } else { 0 }), // Detail
            Constraint::Length(1), // Help
        ])
        .split(f.area());

    draw_header(f, chunks[0]);
    draw_filter_bar(f, app, chunks[1]);
    draw_table(f, app, chunks[2]);

    if app.show_detail {
        draw_detail(f, app, chunks[3]);
    }

    draw_help(f, app, chunks[4]);
}

fn draw_header(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(TEXT_DIM));

    let title = Paragraph::new(Line::from(vec![
        Span::styled("/", Style::default().fg(RED).add_modifier(Modifier::BOLD)),
        Span::styled(
            " Spikes Dashboard ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    ]))
    .block(block);

    f.render_widget(title, area);
}

fn draw_filter_bar(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(20), // Filter input
            Constraint::Min(30),    // Rating buttons
            Constraint::Length(15), // Count
        ])
        .split(area);

    let filter_style = match app.input_mode {
        InputMode::Filter => Style::default().fg(YELLOW),
        InputMode::Normal => Style::default().fg(TEXT_DIM),
    };

    let filter_text = format!(" {} ", app.filter_text);
    let filter_block = Block::default()
        .borders(Borders::ALL)
        .title(" Filter [/] ")
        .border_style(filter_style);

    let filter = Paragraph::new(filter_text).block(filter_block);
    f.render_widget(filter, chunks[0]);

    let rating_buttons = Line::from(vec![
        Span::raw(" Rating: "),
        rating_button("All", app.filter_rating.is_none(), Color::White),
        Span::raw(" "),
        rating_button("1:+", app.filter_rating == Some(Rating::Love), GREEN),
        Span::raw(" "),
        rating_button("2:/", app.filter_rating == Some(Rating::Like), BLUE),
        Span::raw(" "),
        rating_button("3:~", app.filter_rating == Some(Rating::Meh), YELLOW),
        Span::raw(" "),
        rating_button("4:-", app.filter_rating == Some(Rating::No), RED),
    ]);

    let rating_block = Block::default().borders(Borders::ALL).border_style(Style::default().fg(TEXT_DIM));
    let rating = Paragraph::new(rating_buttons).block(rating_block);
    f.render_widget(rating, chunks[1]);

    let count_text = format!(" {}/{} ", app.filtered.len(), app.spikes.len());
    let count_block = Block::default().borders(Borders::ALL).title(" Count ").border_style(Style::default().fg(TEXT_DIM));
    let count = Paragraph::new(count_text).block(count_block);
    f.render_widget(count, chunks[2]);
}

fn rating_button<'a>(label: &'a str, selected: bool, color: Color) -> Span<'a> {
    if selected {
        Span::styled(
            format!("[{}]", label),
            Style::default()
                .fg(color)
                .add_modifier(Modifier::BOLD | Modifier::REVERSED),
        )
    } else {
        Span::styled(format!("[{}]", label), Style::default().fg(color))
    }
}

fn draw_table(f: &mut Frame, app: &App, area: Rect) {
    let header_cells = ["ID", "Type", "Page", "Reviewer", "Rating", "Comments"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(TEXT_MUTED).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).height(1);

    let rows: Vec<Row> = app
        .filtered
        .iter()
        .map(|&idx| {
            let spike = &app.spikes[idx];
            let id = &spike.id[..8.min(spike.id.len())];
            let spike_type = spike.type_str();
            let page = truncate(&spike.page, 15);
            let reviewer = truncate(&spike.reviewer.name, 12);
            let rating = spike.rating_str();
            let comments = truncate(&spike.comments, 30);

            let rating_style = match &spike.rating {
                Some(Rating::Love) => Style::default().fg(GREEN),
                Some(Rating::Like) => Style::default().fg(BLUE),
                Some(Rating::Meh) => Style::default().fg(YELLOW),
                Some(Rating::No) => Style::default().fg(RED),
                None => Style::default().fg(TEXT_DIM),
            };

            Row::new(vec![
                Cell::from(id.to_string()).style(Style::default().fg(TEXT_DIM)),
                Cell::from(spike_type.to_string()),
                Cell::from(page),
                Cell::from(reviewer).style(Style::default().fg(TEXT_MUTED)),
                Cell::from(rating.to_string()).style(rating_style),
                Cell::from(comments),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(10),
        Constraint::Length(8),
        Constraint::Length(17),
        Constraint::Length(14),
        Constraint::Length(8),
        Constraint::Min(20),
    ];

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(" / Spikes ").border_style(Style::default().fg(TEXT_DIM)))
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(20, 20, 23))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("/ ");

    let mut state = TableState::default();
    state.select(Some(app.selected));

    f.render_stateful_widget(table, area, &mut state);
}

fn draw_detail(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Detail [Enter] ")
        .border_style(Style::default().fg(TEXT_DIM));

    if let Some(spike) = app.selected_spike() {
        let selector_line = spike
            .selector
            .as_ref()
            .map(|s| format!("Selector: {}\n", s))
            .unwrap_or_default();

        let element_line = spike
            .element_text
            .as_ref()
            .map(|t| format!("Element: {}\n", truncate(t, 50)))
            .unwrap_or_default();

        let detail_text = format!(
            "ID: {}  |  Page: {}  |  URL: {}\n\
             {}{}\
             Rating: {}  |  Reviewer: {} ({})\n\
             Timestamp: {}\n\
             Comments: {}",
            spike.id,
            spike.page,
            spike.url,
            selector_line,
            element_line,
            spike.rating_str(),
            spike.reviewer.name,
            spike.reviewer.id,
            spike.timestamp,
            spike.comments,
        );

        let paragraph = Paragraph::new(detail_text)
            .block(block)
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, area);
    } else {
        let paragraph = Paragraph::new("No spike selected").style(Style::default().fg(TEXT_DIM)).block(block);
        f.render_widget(paragraph, area);
    }
}

fn draw_help(f: &mut Frame, app: &App, area: Rect) {
    let mode_indicator = match app.input_mode {
        InputMode::Normal => Span::styled("NORMAL", Style::default().fg(GREEN)),
        InputMode::Filter => Span::styled("FILTER", Style::default().fg(YELLOW)),
    };

    let help_text = Line::from(vec![
        Span::raw(" "),
        mode_indicator,
        Span::styled(" | ", Style::default().fg(TEXT_DIM)),
        Span::styled("j/k", Style::default().fg(TEXT_MUTED)),
        Span::styled(":nav ", Style::default().fg(TEXT_DIM)),
        Span::styled("Enter", Style::default().fg(TEXT_MUTED)),
        Span::styled(":detail ", Style::default().fg(TEXT_DIM)),
        Span::styled("/", Style::default().fg(RED)),
        Span::styled(":filter ", Style::default().fg(TEXT_DIM)),
        Span::styled("1-4", Style::default().fg(TEXT_MUTED)),
        Span::styled(":rating ", Style::default().fg(TEXT_DIM)),
        Span::styled("0", Style::default().fg(TEXT_MUTED)),
        Span::styled(":clear ", Style::default().fg(TEXT_DIM)),
        Span::styled("q", Style::default().fg(TEXT_MUTED)),
        Span::styled(":quit", Style::default().fg(TEXT_DIM)),
    ]);

    let help = Paragraph::new(help_text);
    f.render_widget(help, area);
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    } else {
        s.to_string()
    }
}
