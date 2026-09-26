mod app;
mod event;
mod iperf;
mod server;
mod ui;

use anyhow::Result;
use app::{ActivePanel, AppState, InputField, TestMode};
use crossterm::{
    event::{KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use event::{AppEvent, EventHandler};
use iperf::IperfRunner;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, time::Duration};

#[tokio::main]
async fn main() -> Result<()> {
    // Graceful panic cleanup
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        default_hook(panic_info);
    }));

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = AppState::new();
    let mut events = EventHandler::new(Duration::from_millis(100));

    // Spawn server fetcher and auto-pinger task
    let event_tx = events.sender();
    tokio::spawn(async move {
        let raw_servers = server::fetcher::fetch_public_servers().await;
        let sorted_servers = server::pinger::ping_and_sort_servers(raw_servers).await;
        let _ = event_tx.send(AppEvent::ServersLoaded(sorted_servers));
    });

    // Main event loop
    while app.running {
        terminal.draw(|f| ui::render(f, &app))?;

        if let Some(event) = events.next().await {
            match event {
                AppEvent::Key(key) if key.kind == KeyEventKind::Press => {
                    handle_key_event(&mut app, key, &events);
                }
                AppEvent::ServersLoaded(servers) => {
                    app.servers = servers;
                    app.is_loading_servers = false;
                    app.normalize_server_selection();
                    app.update_selected_server_to_inputs();
                    app.current_status = format!(
                        "Loaded {} servers (sorted by latency). Panel 1 populated.",
                        app.servers.len()
                    );
                }
                AppEvent::IperfBandwidth {
                    direction,
                    timestamp_sec,
                    mbps,
                } => {
                    app.add_sample(direction, timestamp_sec, mbps);
                }
                AppEvent::IperfStatus(status) => {
                    app.current_status = status;
                }
                AppEvent::IperfFinished(res) => {
                    app.is_testing = false;
                    match res {
                        Ok(_) => app.current_status = "Test completed successfully.".to_string(),
                        Err(e) => app.current_status = format!("Test error: {}", e),
                    }
                }
                AppEvent::Tick | AppEvent::Key(_) => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn handle_key_event(app: &mut AppState, key: KeyEvent, events: &EventHandler) {
    match key.code {
        KeyCode::Esc => {
            if app.active_panel == ActivePanel::ServerList && app.server_search_active {
                app.server_search_active = false;
                app.server_search.clear();
                app.normalize_server_selection();
            } else {
                app.running = false;
            }
        }
        KeyCode::Tab => app.previous_panel(),
        KeyCode::BackTab => app.next_panel(),
        KeyCode::F(5) => start_test(app, events),
        _ => match app.active_panel {
            ActivePanel::InputSettings => handle_input_panel(app, key, events),
            ActivePanel::ServerList => handle_server_panel(app, key, events),
            ActivePanel::LiveGraph => handle_graph_panel(app, key, events),
        },
    }
}

fn handle_input_panel(app: &mut AppState, key: KeyEvent, events: &EventHandler) {
    match key.code {
        KeyCode::Up => app.active_field = app.active_field.prev(),
        KeyCode::Down => app.active_field = app.active_field.next(),
        KeyCode::Char('q') if app.active_field == InputField::Mode => app.running = false,
        KeyCode::Char('s') if app.active_field == InputField::Mode => start_test(app, events),
        KeyCode::Enter if app.active_field == InputField::Mode => {
            app.config.mode = app.config.mode.next();
        }
        KeyCode::Char(' ') if app.active_field == InputField::Mode => {
            app.config.mode = app.config.mode.next();
        }
        KeyCode::Backspace => match app.active_field {
            InputField::Host => {
                app.custom_host.pop();
            }
            InputField::Port => {
                app.custom_port.pop();
            }
            InputField::Duration => {
                app.duration_input.pop();
                app.sync_config_from_inputs();
            }
            InputField::Streams => {
                app.streams_input.pop();
                app.sync_config_from_inputs();
            }
            InputField::Mode => {}
        },
        KeyCode::Char(c) => match app.active_field {
            InputField::Host => {
                app.custom_host.push(c);
            }
            InputField::Port => {
                if c.is_ascii_digit() {
                    app.custom_port.push(c);
                }
            }
            InputField::Duration => {
                if c.is_ascii_digit() {
                    app.duration_input.push(c);
                    app.sync_config_from_inputs();
                }
            }
            InputField::Streams => {
                if c.is_ascii_digit() {
                    app.streams_input.push(c);
                    app.sync_config_from_inputs();
                }
            }
            InputField::Mode => {
                app.config.mode = match c.to_ascii_lowercase() {
                    'd' => TestMode::Download,
                    'u' => TestMode::Upload,
                    'b' => TestMode::Both,
                    _ => app.config.mode,
                };
            }
        },
        _ => {}
    }
}

fn handle_server_panel(app: &mut AppState, key: KeyEvent, events: &EventHandler) {
    if app.server_search_active {
        match key.code {
            KeyCode::Enter => app.server_search_active = false,
            KeyCode::Backspace => {
                app.server_search.pop();
                app.normalize_server_selection();
            }
            KeyCode::Char(c) => {
                app.server_search.push(c);
                app.normalize_server_selection();
            }
            _ => {}
        }
        return;
    }

    match key.code {
        KeyCode::Char('/') => app.server_search_active = true,
        KeyCode::Up | KeyCode::Char('k') => app.previous_server(),
        KeyCode::Down | KeyCode::Char('j') => app.next_server(),
        KeyCode::Char('s') => start_test(app, events),
        KeyCode::Char('q') => app.running = false,
        _ => {}
    }
}

fn handle_graph_panel(app: &mut AppState, key: KeyEvent, events: &EventHandler) {
    match key.code {
        KeyCode::Char('s') | KeyCode::Enter => start_test(app, events),
        KeyCode::Char('q') => app.running = false,
        _ => {}
    }
}

fn start_test(app: &mut AppState, events: &EventHandler) {
    if !app.is_testing {
        app.is_testing = true;
        app.reset_test_stats();

        let (host, port) = app.get_selected_target();
        let config = app.config.clone();
        let tx = events.sender();

        tokio::spawn(async move {
            if let Err(e) = IperfRunner::run(host, port, config, tx.clone()).await {
                let _ = tx.send(AppEvent::IperfFinished(Err(e.to_string())));
            }
        });
    }
}
