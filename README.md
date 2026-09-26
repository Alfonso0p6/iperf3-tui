# iperf3-tui

A terminal user interface for running and comparing `iperf3` throughput tests.

The application loads public `iperf3` servers, measures their TCP latency, lets you search the catalog, and displays live throughput metrics in the terminal.

## Features

- Public server catalog with latency sorting and fallback servers
- Search across host, port, options, bandwidth, continent, country, site, provider, and latency
- Upload, download, or combined download-then-upload tests
- Live chart with separate Download and Upload series
- Separate current, peak, and average throughput metrics
- Terminal-friendly colors with no application background fill
- Exact `iperf3` error output in the status bar

## Requirements

- Rust stable and Cargo
- `iperf3` available on `PATH`
- A terminal with Unicode support

Install `iperf3` using your operating system's package manager. For example:

```bash
# Debian/Ubuntu
sudo apt install iperf3

# Fedora
sudo dnf install iperf3

# Arch Linux
sudo pacman -S iperf3
```

## Run

```bash
cargo run --release
```

The server catalog is fetched from `export.iperf3serverlist.net`. If the request fails, the application uses a small built-in fallback list.

## Controls

| Key | Action |
| --- | --- |
| `Tab` | Switch to the previous panel |
| `Shift+Tab` | Switch to the next panel |
| `Up` / `Down` | Move selection |
| `/` | Search servers |
| `Enter` | Change test mode in panel 2 or start a test in panel 3 |
| `Space` | Change test mode in panel 2 |
| `d` / `u` / `b` | Select download / upload / both in panel 2 |
| `F5` | Start a test from any panel |
| `Esc` | Exit search or quit |

## Project Layout

- `src/app.rs`: application state, navigation, filtering, and throughput statistics
- `src/event.rs`: terminal and application event handling
- `src/iperf/runner.rs`: `iperf3` process execution and output parsing
- `src/server/fetcher.rs`: remote catalog parsing and fallback data
- `src/server/pinger.rs`: latency measurement and server sorting
- `src/ui.rs`: Ratatui rendering and terminal palette

## Development

```bash
cargo fmt -- --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

## License

This project does not currently declare a license. Add one before distributing it publicly.
