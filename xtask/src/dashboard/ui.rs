use crate::dashboard::state::{DashboardState, ErrorStats, EventSeverity, RecentEvent};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, List, ListItem, Paragraph, Row, Table, Tabs, Wrap},
    Frame,
};

/// Render the main dashboard UI
pub fn render_dashboard(
    frame: &mut Frame<CrosstermBackend<std::io::Stdout>>,
    state: &DashboardState,
) {
    let size = frame.size();

    // Create main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(1), // Tabs
            Constraint::Min(0),    // Content
            Constraint::Length(3), // Footer
        ])
        .split(size);

    // Header
    render_header(frame, chunks[0], state);

    // Tabs
    render_tabs(frame, chunks[1], state);

    // Content based on selected tab
    let content_chunk = chunks[2];
    match state.selected_tab {
        0 => render_overview_tab(frame, content_chunk, state),
        1 => render_errors_tab(frame, content_chunk, state),
        2 => render_events_tab(frame, content_chunk, state),
        3 => render_stats_tab(frame, content_chunk, state),
        _ => render_overview_tab(frame, content_chunk, state),
    }

    // Footer
    render_footer(frame, chunks[3]);
}

/// Render the header section
fn render_header(
    frame: &mut Frame<CrosstermBackend<std::io::Stdout>>,
    area: Rect,
    state: &DashboardState,
) {
    let header = Paragraph::new(vec![
        Line::from(vec![Span::styled(
            "🚀 Hooksmith Event-Driven Dashboard",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::raw("Status: "),
            Span::styled(
                if state.system_status.is_running {
                    "🟢 Active"
                } else {
                    "🔴 Stopped"
                },
                Style::default().fg(if state.system_status.is_running {
                    Color::Green
                } else {
                    Color::Red
                }),
            ),
            Span::raw(" | Auto-push: "),
            Span::styled(
                if state.system_status.auto_push_enabled {
                    "🟢 On"
                } else {
                    "🔴 Off"
                },
                Style::default().fg(if state.system_status.auto_push_enabled {
                    Color::Green
                } else {
                    Color::Red
                }),
            ),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL).title("Dashboard"));
    frame.render_widget(header, area);
}

/// Render the tabs
fn render_tabs(
    frame: &mut Frame<CrosstermBackend<std::io::Stdout>>,
    area: Rect,
    state: &DashboardState,
) {
    let tabs = Tabs::new(vec!["Overview", "Errors", "Events", "Stats"])
        .block(Block::default().borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .select(state.selected_tab);
    frame.render_widget(tabs, area);
}

/// Render the overview tab
fn render_overview_tab(
    frame: &mut Frame<CrosstermBackend<std::io::Stdout>>,
    area: Rect,
    state: &DashboardState,
) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left side - System Status
    render_system_status(frame, chunks[0], state);

    // Right side - Quick Stats
    render_quick_stats(frame, chunks[1], state);
}

/// Render system status
fn render_system_status(
    frame: &mut Frame<CrosstermBackend<std::io::Stdout>>,
    area: Rect,
    state: &DashboardState,
) {
    let status_text = vec![
        Line::from(vec![
            Span::raw("Uptime: "),
            Span::styled(
                format!("{:?}", state.system_status.uptime),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::raw("File Watch: "),
            Span::styled(
                if state.system_status.file_watch_enabled {
                    "🟢 On"
                } else {
                    "🔴 Off"
                },
                Style::default().fg(if state.system_status.file_watch_enabled {
                    Color::Green
                } else {
                    Color::Red
                }),
            ),
        ]),
        Line::from(vec![
            Span::raw("Heartbeat: "),
            Span::styled(
                format!("{}s", state.system_status.heartbeat_interval),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(vec![
            Span::raw("Last Update: "),
            Span::styled(
                state.last_update.format("%H:%M:%S").to_string(),
                Style::default().fg(Color::Magenta),
            ),
        ]),
    ];

    let status_widget = Paragraph::new(status_text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("System Status"),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(status_widget, area);
}

/// Render quick stats
fn render_quick_stats(
    frame: &mut Frame<CrosstermBackend<std::io::Stdout>>,
    area: Rect,
    state: &DashboardState,
) {
    let stats_text = vec![
        Line::from(vec![
            Span::raw("Active Errors: "),
            Span::styled(
                format!("{}", state.errors.len()),
                Style::default().fg(if state.errors.is_empty() {
                    Color::Green
                } else {
                    Color::Red
                }),
            ),
        ]),
        Line::from(vec![
            Span::raw("Total Events: "),
            Span::styled(
                format!("{}", state.event_stats.total_events),
                Style::default().fg(Color::Blue),
            ),
        ]),
        Line::from(vec![
            Span::raw("Error Count: "),
            Span::styled(
                format!("{}", state.event_stats.errors_count),
                Style::default().fg(Color::Red),
            ),
        ]),
        Line::from(vec![
            Span::raw("Info Count: "),
            Span::styled(
                format!("{}", state.event_stats.info_count),
                Style::default().fg(Color::Green),
            ),
        ]),
    ];

    let stats_widget = Paragraph::new(stats_text)
        .block(Block::default().borders(Borders::ALL).title("Quick Stats"))
        .wrap(Wrap { trim: true });
    frame.render_widget(stats_widget, area);
}

/// Render errors tab
fn render_errors_tab(
    frame: &mut Frame<CrosstermBackend<std::io::Stdout>>,
    area: Rect,
    state: &DashboardState,
) {
    if state.errors.is_empty() {
        let no_errors = Paragraph::new("✅ No errors detected")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Active Errors"),
            )
            .style(Style::default().fg(Color::Green));
        frame.render_widget(no_errors, area);
        return;
    }

    // Create table rows for errors
    let rows: Vec<Row> = state
        .errors
        .values()
        .take(20) // Limit to 20 errors for display
        .map(|error| {
            let location = if let (Some(file), Some(line)) = (error.file.as_ref(), error.line) {
                format!("{}:{}", file, line)
            } else {
                "Unknown".to_string()
            };

            Row::new(vec![
                Cell::from(error.error_type.clone()),
                Cell::from(format!("{}x", error.count)),
                Cell::from(location),
                Cell::from(if error.message.len() > 30 {
                    format!("{}...", &error.message[..27])
                } else {
                    error.message.clone()
                }),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        &[
            Constraint::Percentage(20),
            Constraint::Percentage(10),
            Constraint::Percentage(20),
            Constraint::Percentage(50),
        ],
    )
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title("Active Errors"),
    )
    .style(Style::default().fg(Color::White))
    .header(
        Row::new(vec!["Type", "Count", "Location", "Message"]).style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    );

    frame.render_widget(table, area);
}

/// Render events tab
fn render_events_tab(
    frame: &mut Frame<CrosstermBackend<std::io::Stdout>>,
    area: Rect,
    state: &DashboardState,
) {
    if state.recent_events.is_empty() {
        let no_events = Paragraph::new("📡 No events recorded yet")
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Recent Events"),
            )
            .style(Style::default().fg(Color::Blue));
        frame.render_widget(no_events, area);
        return;
    }

    // Create list items for recent events
    let items: Vec<ListItem> = state
        .recent_events
        .iter()
        .rev() // Show newest first
        .take(15) // Limit to 15 events
        .map(|event| {
            let severity_color = match event.severity {
                EventSeverity::Error => Color::Red,
                EventSeverity::Warning => Color::Yellow,
                EventSeverity::Info => Color::Green,
            };

            let time_str = event.timestamp.format("%H:%M:%S").to_string();
            let content = format!(
                "[{}] {}: {} - {}",
                time_str, event.actor, event.event_type, event.message
            );

            ListItem::new(vec![Line::from(vec![Span::styled(
                content,
                Style::default().fg(severity_color),
            )])])
        })
        .collect();

    let events_list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Recent Events"),
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(events_list, area);
}

/// Render stats tab
fn render_stats_tab(
    frame: &mut Frame<CrosstermBackend<std::io::Stdout>>,
    area: Rect,
    state: &DashboardState,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(0)])
        .split(area);

    // Top section - Event type breakdown
    render_event_type_stats(frame, chunks[0], state);

    // Bottom section - Actor breakdown
    render_actor_stats(frame, chunks[1], state);
}

/// Render event type statistics
fn render_event_type_stats(
    frame: &mut Frame<CrosstermBackend<std::io::Stdout>>,
    area: Rect,
    state: &DashboardState,
) {
    if state.event_stats.events_by_type.is_empty() {
        let no_stats = Paragraph::new("📊 No event statistics available")
            .block(Block::default().borders(Borders::ALL).title("Event Types"))
            .style(Style::default().fg(Color::Blue));
        frame.render_widget(no_stats, area);
        return;
    }

    // Create table rows for event types
    let mut event_type_rows: Vec<Row> = state
        .event_stats
        .events_by_type
        .iter()
        .map(|(event_type, count)| {
            Row::new(vec![
                Cell::from(event_type.clone()),
                Cell::from(count.to_string()),
            ])
        })
        .collect();

    // Sort by count (descending)
    event_type_rows.sort_by(|a, b| {
        let count_a: u64 = a.cells[1].content().parse().unwrap_or(0);
        let count_b: u64 = b.cells[1].content().parse().unwrap_or(0);
        count_b.cmp(&count_a)
    });

    let table = Table::new(
        event_type_rows,
        &[Constraint::Percentage(70), Constraint::Percentage(30)],
    )
    .block(Block::default().borders(Borders::ALL).title("Event Types"))
    .style(Style::default().fg(Color::White))
    .header(
        Row::new(vec!["Event Type", "Count"]).style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    );

    frame.render_widget(table, area);
}

/// Render actor statistics
fn render_actor_stats(
    frame: &mut Frame<CrosstermBackend<std::io::Stdout>>,
    area: Rect,
    state: &DashboardState,
) {
    if state.event_stats.events_by_actor.is_empty() {
        let no_stats = Paragraph::new("👥 No actor statistics available")
            .block(Block::default().borders(Borders::ALL).title("Actors"))
            .style(Style::default().fg(Color::Blue));
        frame.render_widget(no_stats, area);
        return;
    }

    // Create table rows for actors
    let mut actor_rows: Vec<Row> = state
        .event_stats
        .events_by_actor
        .iter()
        .map(|(actor, count)| {
            Row::new(vec![
                Cell::from(actor.clone()),
                Cell::from(count.to_string()),
            ])
        })
        .collect();

    // Sort by count (descending)
    actor_rows.sort_by(|a, b| {
        let count_a: u64 = a.cells[1].content().parse().unwrap_or(0);
        let count_b: u64 = b.cells[1].content().parse().unwrap_or(0);
        count_b.cmp(&count_a)
    });

    let table = Table::new(
        actor_rows,
        &[Constraint::Percentage(70), Constraint::Percentage(30)],
    )
    .block(Block::default().borders(Borders::ALL).title("Actors"))
    .style(Style::default().fg(Color::White))
    .header(
        Row::new(vec!["Actor", "Events"]).style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
    );

    frame.render_widget(table, area);
}

/// Render the footer
fn render_footer(frame: &mut Frame<CrosstermBackend<std::io::Stdout>>, area: Rect) {
    let footer = Paragraph::new(vec![Line::from(vec![
        Span::raw("💡 Press "),
        Span::styled("Ctrl+C", Style::default().fg(Color::Yellow)),
        Span::raw(" to stop | "),
        Span::raw("Tab: "),
        Span::styled("1-4", Style::default().fg(Color::Yellow)),
        Span::raw(" to switch tabs | "),
        Span::raw("←/→: "),
        Span::styled("Navigate", Style::default().fg(Color::Yellow)),
    ])])
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, area);
}
