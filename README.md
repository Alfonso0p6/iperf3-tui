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

- `iperf3` available on system `PATH`
- A terminal with Unicode support

Install `iperf3` using your operating system's package manager. For example:

```bash
# Debian/Ubuntu
sudo apt install iperf3

# Fedora
sudo dnf install iperf3

# Arch Linux
sudo pacman -S iperf3

# macOS
brew install iperf3
```

## Installation

### Fast One-Line Installers (Pre-compiled Binaries)

You can install `iperf3-tui` instantly using the pre-compiled binaries from the latest release:

#### Linux & macOS (Shell)
```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Alfonso0p6/iperf3-tui/releases/latest/download/iperf3-tui-installer.sh | sh
```

#### Windows (PowerShell)
```powershell
irm https://github.com/Alfonso0p6/iperf3-tui/releases/latest/download/iperf3-tui-installer.ps1 | iex
```

---

### From GitHub Releases

You can manually download the standalone binary for your architecture from the [Releases Page](https://github.com/Alfonso0p6/iperf3-tui/releases):
- **Linux (`x86_64-unknown-linux-musl`)**: Fully static binary (works on Alpine, Ubuntu, Debian, Arch, Fedora, etc.)
- **Linux ARM64 (`aarch64-unknown-linux-gnu`)**: Raspberry Pi and ARM cloud instances
- **macOS Apple Silicon (`aarch64-apple-darwin`)**: M1/M2/M3/M4 Macs
- **Windows (`x86_64-pc-windows-msvc`)**: 64-bit Windows systems

---

### Build from Source

If you have Rust and Cargo installed:

```bash
# Clone and run directly
git clone [https://github.com/Alfonso0p6/iperf3-tui.git](https://github.com/Alfonso0p6/iperf3-tui.git)
cd iperf3-tui
cargo run --release

# Or install binary locally
cargo install --path .
```

## Usage

Once installed, you can launch the application directly from your terminal:

```bash
iperf3-tui
```

### Workflow

1. **Startup**: The server catalog is fetched from `export.iperf3serverlist.net`. If the request fails, the application uses a small built-in fallback list. The app will automatically measure TCP latency for the available servers.
2. **Select a Server**: Use the `Up`/`Down` arrows to navigate the server list, or press `/` to open the search bar and filter by country, provider, or latency.
3. **Configure Test**: Press `Tab` to switch focus to the configuration panel. Press `Space` or `d`/`u`/`b` to cycle between Download, Upload, or Both (Download then Upload).
4. **Start**: Press `F5` or `Enter` to execute the test. The bottom panel will render the live chart and throughput statistics.

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

## Uninstallation

To remove `iperf3-tui` from your system:

### Linux / macOS
```bash
# If installed via the standard installer script
rm ~/.local/bin/iperf3-tui

# If installed via Cargo
rm ~/.cargo/bin/iperf3-tui
```

### Windows (PowerShell)
```powershell
Remove-Item "$env:LOCALAPPDATA\iperf3-tui\iperf3-tui.exe" -ErrorAction SilentlyContinue
Remove-Item "$env:USERPROFILE\.cargo\bin\iperf3-tui.exe" -ErrorAction SilentlyContinue
```

### Cargo
If you installed it via `cargo install`:
```bash
cargo uninstall iperf3-tui
```

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

Distributed under the GNU General Public License v3.0. See [LICENSE](LICENSE.txt) for more information.
