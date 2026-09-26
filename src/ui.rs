use crate::app::{ActivePanel, AppState, InputField, TestMode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{
        Axis, Block, Borders, Chart, Dataset, GraphType, List, ListItem, ListState, Paragraph,
    },
    Frame,
};

struct Palette {
    border: Color,
    active: Color,
    primary: Color,
    muted: Color,
    selection: Color,
    download: Color,
    upload: Color,
    warning: Color,
}

const PALETTE: Palette = Palette {
    border: Color::DarkGray,
    active: Color::Green,
    primary: Color::Cyan,
    muted: Color::Gray,
    selection: Color::Yellow,
    download: Color::LightCyan,
    upload: Color::LightMagenta,
    warning: Color::Yellow,
};

pub fn render(f: &mut Frame, app: &AppState) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(12),
            Constraint::Length(3),
        ])
        .split(f.area());

    let header = Paragraph::new(" iPerf3 TUI | [Tab] Switch Panel | [Up/Down] Navigate | [Space/Enter] Edit/Change | [F5 / s] Start Test | [Esc] Exit")
        .style(Style::default().fg(PALETTE.primary).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).title(" App "));
    f.render_widget(header, main_chunks[0]);

    let middle_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(34),
            Constraint::Percentage(32),
            Constraint::Percentage(34),
        ])
        .split(main_chunks[1]);

    render_servers_panel(f, app, middle_chunks[0]);
    render_input_panel(f, app, middle_chunks[1]);
    render_graph_panel(f, app, middle_chunks[2]);

    let footer = Paragraph::new(format!(" Status: {}", app.current_status))
        .style(Style::default().fg(if app.is_testing {
            PALETTE.active
        } else {
            PALETTE.warning
        }))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Console Status "),
        );
    f.render_widget(footer, main_chunks[2]);
}

fn render_servers_panel(f: &mut Frame, app: &AppState, area: Rect) {
    let border_color = if app.active_panel == ActivePanel::ServerList {
        PALETTE.active
    } else {
        PALETTE.border
    };

    if app.is_loading_servers {
        let loading = Paragraph::new("\n  Pinging and loading servers...")
            .style(Style::default().fg(PALETTE.warning))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" [1] Public Servers ")
                    .border_style(Style::default().fg(border_color)),
            );
        f.render_widget(loading, area);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(area);

    let search_style = if app.server_search_active {
        Style::default()
            .fg(PALETTE.selection)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(PALETTE.muted)
    };
    let search = Paragraph::new(format!(" / {}", app.server_search))
        .style(search_style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Search All Fields "),
        );
    f.render_widget(search, chunks[0]);

    let visible_indices = app.filtered_server_indices();
    let items: Vec<ListItem> = visible_indices
        .iter()
        .map(|&idx| {
            let s = &app.servers[idx];
            let selected_mark = if idx == app.selected_server_idx {
                "> "
            } else {
                "  "
            };
            let ping_str = s.ping_ms.map_or("N/A".to_string(), |p| format!("{}ms", p));
            let line = format!(
                "{}{:<18} {:<11} {} {} {}",
                selected_mark, s.host, s.port_display, s.country, s.site, ping_str
            );
            let style = if idx == app.selected_server_idx {
                Style::default()
                    .fg(PALETTE.selection)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(line).style(style)
        })
        .collect();

    let items = if items.is_empty() {
        vec![ListItem::new("  No servers found")]
    } else {
        items
    };

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(
                " [1] Servers ({}/{}) ",
                visible_indices.len(),
                app.servers.len()
            ))
            .border_style(Style::default().fg(border_color)),
    );
    let mut list_state = ListState::default();
    list_state.select(
        visible_indices
            .iter()
            .position(|&idx| idx == app.selected_server_idx),
    );
    f.render_stateful_widget(list, chunks[1], &mut list_state);
}

fn render_input_panel(f: &mut Frame, app: &AppState, area: Rect) {
    let is_active = app.active_panel == ActivePanel::InputSettings;
    let border_color = if is_active {
        PALETTE.active
    } else {
        PALETTE.border
    };

    let fields = [
        (InputField::Host, "Target Host ", &app.custom_host),
        (InputField::Port, "Target Port ", &app.custom_port),
        (
            InputField::Duration,
            "Duration (-t) ",
            &format!("{} s", app.duration_input),
        ),
        (InputField::Streams, "Streams (-P)", &app.streams_input),
    ];

    let mut lines = Vec::new();
    lines.push(String::from(""));

    for (field, label, value) in fields {
        let is_selected = is_active && app.active_field == field;
        let prefix = if is_selected { "> " } else { "  " };
        lines.push(format!("{}{:<12}: {}", prefix, label, value));
    }

    let mode_selected = is_active && app.active_field == InputField::Mode;
    let mode_prefix = if mode_selected { "> " } else { "  " };
    lines.push(format!("{} Mode: {}", mode_prefix, app.config.mode.label()));

    lines.push(String::from("\n --- Panel 2 Help ---"));
    lines.push(String::from(" Up/Down : Select field"));
    lines.push(String::from(" Type    : Edit text"));
    lines.push(String::from(" Space   : Change mode (d/u/b)"));

    let panel = Paragraph::new(lines.join("\n")).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" [2] Target & Config ")
            .border_style(Style::default().fg(border_color)),
    );
    f.render_widget(panel, area);
}

fn render_graph_panel(f: &mut Frame, app: &AppState, area: Rect) {
    let border_color = if app.active_panel == ActivePanel::LiveGraph {
        PALETTE.active
    } else {
        PALETTE.border
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(9), Constraint::Min(3)])
        .split(area);

    let mode_str = app.config.mode.label();
    let mode_color = match app.config.mode {
        TestMode::Download => PALETTE.download,
        TestMode::Upload => PALETTE.upload,
        TestMode::Both => PALETTE.primary,
    };
    if app.config.mode == TestMode::Both {
        let metric_columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[0]);
        let download_metrics = Paragraph::new(format!(
            " Current: {:.2} Mbps\n Peak: {:.2} Mbps\n Average: {:.2} Mbps",
            app.download_current_mbps, app.download_peak_mbps, app.download_avg_mbps
        ))
        .style(
            Style::default()
                .fg(PALETTE.download)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Download ")
                .border_style(Style::default().fg(border_color)),
        );
        let upload_metrics = Paragraph::new(format!(
            " Current: {:.2} Mbps\n Peak: {:.2} Mbps\n Average: {:.2} Mbps",
            app.upload_current_mbps, app.upload_peak_mbps, app.upload_avg_mbps
        ))
        .style(
            Style::default()
                .fg(PALETTE.upload)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Upload ")
                .border_style(Style::default().fg(border_color)),
        );
        f.render_widget(download_metrics, metric_columns[0]);
        f.render_widget(upload_metrics, metric_columns[1]);
    } else {
        let metrics = Paragraph::new(format!(
            " Mode: {}\n Current: {:.2} Mbps\n Peak: {:.2} Mbps\n Average: {:.2} Mbps",
            mode_str, app.current_mbps, app.peak_mbps, app.avg_mbps
        ))
        .style(Style::default().fg(mode_color).add_modifier(Modifier::BOLD))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" [3] Numeric Metrics ")
                .border_style(Style::default().fg(border_color)),
        );
        f.render_widget(metrics, chunks[0]);
    }

    let datasets = if app.config.mode == TestMode::Both {
        vec![
            Dataset::default()
                .name("Download")
                .marker(ratatui::symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(
                    Style::default()
                        .fg(PALETTE.download)
                        .add_modifier(Modifier::BOLD),
                )
                .data(&app.download_samples),
            Dataset::default()
                .name("Upload")
                .marker(ratatui::symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(
                    Style::default()
                        .fg(PALETTE.upload)
                        .add_modifier(Modifier::UNDERLINED),
                )
                .data(&app.upload_samples),
        ]
    } else {
        vec![Dataset::default()
            .name(mode_str)
            .marker(ratatui::symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(mode_color))
            .data(&app.bandwidth_samples)]
    };

    let max_x = (app.config.duration_secs as f64).max(1.0);
    let max_y = app.max_bandwidth_found.max(10.0);

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Live Throughput ")
                .border_style(Style::default().fg(border_color)),
        )
        .x_axis(
            Axis::default()
                .title("Sec")
                .style(Style::default().fg(PALETTE.muted))
                .bounds([0.0, max_x]),
        )
        .y_axis(
            Axis::default()
                .title("Mbps")
                .style(Style::default().fg(PALETTE.muted))
                .bounds([0.0, max_y]),
        );

    f.render_widget(chart, chunks[1]);
}
