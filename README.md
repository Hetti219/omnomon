# omnomon

A unified terminal system monitor for Linux laptops with NVIDIA GPUs. One screen, every metric — CPU, GPU, memory, disk, network, processes, battery, and thermals — without juggling `htop`, `nvtop`, `iotop`, and `acpi` in five different tmux panes.

Built in Rust with [`ratatui`](https://ratatui.rs) + [`crossterm`](https://github.com/crossterm-rs/crossterm) and [`nvml-wrapper`](https://github.com/Cldfire/nvml-wrapper) for first-class NVIDIA support.

## Features

- **Dashboard** — every metric at a glance, two-column on wide terminals, stacked on narrow ones.
- **CPU** — per-core usage / frequency / temperature, package temp, history graph.
- **GPU (NVIDIA)** — utilization, VRAM, power draw vs. limit, clocks, encoder/decoder load, per-process VRAM.
- **Memory** — RAM and swap with cached/buffers breakdown and history graph.
- **Disk** — per-mount usage and live read/write throughput with history.
- **Network** — per-interface throughput, IPv4/IPv6, history; cycle interfaces with `n`.
- **Processes** — sortable, filterable, kill/SIGKILL from the UI; merges GPU memory per PID.
- **Battery** — charge / state / health / cycles / voltage / temp, charge and power-draw history graphs.
- **Thermal** — every `/sys/class/thermal` zone with critical-trip-relative bars, plus fan RPMs.
- **Themes** — `default`, `gruvbox`, `dracula`, `nord`, `catppuccin`, `solarized`.
- **Graceful degradation** — no NVIDIA GPU? Builds and runs without it. No battery? GPU panel and battery tab show a friendly placeholder. Missing sysfs files don't crash anything.

## Install

### Prerequisites

- Rust toolchain (1.75+ recommended) — install via [rustup](https://rustup.rs).
- For GPU monitoring: an NVIDIA driver with NVML (bundled with the proprietary driver — no extra package needed).
- A terminal that supports Unicode and 24-bit color for the best experience.

### From source

```bash
git clone <repo-url> omnomon
cd omnomon
cargo install --path .
```

The binary is installed to `~/.cargo/bin/omnomon` (~3–5 MB after LTO/strip).

### Build without NVIDIA support

If you don't have an NVIDIA GPU or don't want NVML linkage:

```bash
cargo build --release --no-default-features
```

## Usage

```
omnomon [OPTIONS]
```

| Flag | Description |
|------|-------------|
| `-r, --rate <MS>` | Refresh rate in ms (250–5000, default 1000) |
| `-t, --theme <NAME>` | Color theme |
| `-c, --config <PATH>` | Path to config file |
| `--no-gpu` | Disable GPU monitoring at runtime |
| `--fahrenheit` | Show temperatures in °F |
| `-v, --verbose` | Debug logging to `/tmp/omnomon.log` |

CLI flags override config-file values.

## Keybindings

| Key | Action | Context |
|-----|--------|---------|
| `q` / `Ctrl+C` | Quit | Global |
| `?` | Toggle help overlay | Global |
| `Esc` | Close help / clear filter | Global |
| `1`–`9` | Switch tab | Global |
| `Tab` / `Shift+Tab` | Next / previous tab | Global |
| `r` | Force refresh | Global |
| `+` / `-` | Zoom graph time window in / out | Graph views |
| `n` | Cycle network interface | Network tab |
| `↑` `↓` / `k` `j` | Move selection | Processes |
| `Home` `End` / `g` `G` | Jump to top / bottom | Processes |
| `/` | Edit filter (Enter to apply, Esc to clear) | Processes |
| `s` | Cycle sort column | Processes |
| `S` | Toggle sort direction | Processes |
| `K` | Kill selected (SIGTERM) | Processes |
| `D` | Kill selected (SIGKILL) | Processes |
| `t` | Toggle tree view | Processes |

## Configuration

omnomon reads `$XDG_CONFIG_HOME/omnomon/config.toml` (typically `~/.config/omnomon/config.toml`). All keys are optional — defaults are used for anything missing.

```toml
[general]
refresh_rate_ms = 1000        # 250–5000
temperature_unit = "celsius"  # "celsius" or "fahrenheit"
default_tab = "dashboard"     # dashboard | cpu | gpu | memory | disk | network | processes | battery | thermal
graph_time_window = "60s"     # "30s" | "60s" | "5m"

[theme]
name = "default"              # default | gruvbox | dracula | nord | catppuccin | solarized

[network]
default_interface = "auto"

[process]
default_sort = "cpu"          # cpu | memory | pid | name | gpu
show_gpu_column = true

[dashboard]
show_battery = true
show_thermal = true
show_disk = true
```

## Building & contributing

```bash
cargo run                       # debug build, run
cargo build --release           # optimized binary at target/release/omnomon
cargo test                      # run the unit-test suite
cargo test parse_minimal_toml   # run a single test
cargo clippy --all-targets      # lint
cargo fmt                       # format
```

A high-level architectural overview lives in [CLAUDE.md](CLAUDE.md); the original design document and feature spec is in [omnomon-guide.md](omnomon-guide.md).

If you're adding a new metric, the pattern is:

1. Add a collector in [`src/collector/`](src/collector/) implementing the `Collector` trait. Return `None` or an empty snapshot when the underlying hardware/sysfs file is missing — never panic.
2. Add the snapshot to `SystemSnapshot` in [`src/state.rs`](src/state.rs) and a matching field on `AppState` if you need history.
3. Wire it into `App::tick` in [`src/app.rs`](src/app.rs).
4. Add a renderer module under [`src/ui/`](src/ui/) and a tab entry in [`src/ui/mod.rs`](src/ui/mod.rs).

All NVML usage must stay behind `#[cfg(feature = "nvidia")]` so `--no-default-features` builds keep working.

## License

MIT — see [Cargo.toml](Cargo.toml). Pull requests, bug reports, and feature ideas are welcome.
