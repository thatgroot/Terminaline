# ⚡ Terminaline

A comprehensive, real-time macOS system inspector built with [Ratatui](https://ratatui.rs). Navigate 10 interactive tabs covering every corner of your hardware — from RAM pressure and CPU topology to live network connections and disk file browsing.

![macOS](https://img.shields.io/badge/platform-macOS_ARM64-blue) ![Rust](https://img.shields.io/badge/language-Rust-orange) ![TUI](https://img.shields.io/badge/interface-Terminal_UI-green)

![Terminaline Preview](preview.png)

---

## Features

| Tab | Key | What It Shows |
|-----|-----|---------------|
| **RAM** | `1` | Physical/virtual memory, VM page stats, swap, compression, memory pressure gauge, top 10 memory consumers |
| **Map** | `2` | Process memory regions with cursor selection — address range, size, permissions (R/W/X), region type, full path |
| **Visual** | `3` | Address-space block map with color-coded region types, memory breakdown bars, selected-region highlight |
| **CPU** | `4` | Brand, architecture, P/E core topology, cache hierarchy (L1i/L1d/L2/L3), ARM64 feature flags, register architecture, per-core usage bars |
| **Disk** | `5` | Hardware info, SMART status, I/O rates, partition list with usage gauges — **Enter** to browse files with sort & system-file filter |
| **Network** | `6` | Lifetime packet/byte totals, per-interface details (IP, MTU, status, errors) |
| **GPU** | `7` | Chipset, Metal version, cores, vendor, display info, Unified Memory Architecture |
| **Battery** | `8` | Charge gauge, state, cycle count, health %, voltage, current, temperature |
| **Camera** | `9` | Connected cameras with model ID and unique ID |
| **Activity** | `0` | **Live** per-process network connections — PID, protocol, local/remote addresses, state, with connection detail panel |

## Keyboard Controls

| Key | Action |
|-----|--------|
| `1`–`9`, `0` | Jump to tab |
| `Tab` / `Shift+Tab` | Next / Previous tab |
| `←` / `→` | Previous / Next tab (context-aware in Disk file browser) |
| `↑` / `↓` / `j` / `k` | Scroll or move cursor |
| `PgUp` / `PgDn` | Jump 10–20 items |
| `Enter` | Open partition (Disk) / Navigate into folder |
| `Esc` | Go back (Disk file browser) / Quit |
| `s` | Cycle sort mode (Disk files: size↓, size↑, name A→Z, name Z→A) |
| `f` | Toggle system-file filter (Disk files) |
| `q` | Quit |

## Data Sources

All data is collected from native macOS APIs and CLI tools — no third-party daemons required:

- **`sysctl`** — CPU topology, cache sizes, ARM64 features, physical memory
- **`vm_stat`** — VM page statistics, compressions, faults
- **`top -l1`** — Process stats, CPU usage, disk I/O, network totals
- **`iostat`** — Disk transfer rates
- **`diskutil`** — Disk hardware, SMART status
- **`netstat`** — Per-interface network stats
- **`ifconfig`** — Interface IP and status
- **`lsof -i`** — Live per-process network connections
- **`pmset -g batt`** / **`ioreg`** — Battery details
- **`system_profiler`** — GPU, camera hardware info
- **`proc-maps`** crate — Process memory region mapping

## Requirements

- **macOS** (ARM64 / Apple Silicon recommended)
- **Rust** 1.70+ with Cargo

## Build & Run

```bash
# Clone
git clone <repo-url>
cd memory_visualizer

# Run (debug)
cargo run

# Build optimized release binary
cargo build --release --target aarch64-apple-darwin

# Run the release binary
./target/aarch64-apple-darwin/release/terminaline
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| [`ratatui`](https://ratatui.rs) | Terminal UI framework |
| [`crossterm`](https://docs.rs/crossterm) | Terminal raw mode, keyboard events |
| [`sysinfo`](https://docs.rs/sysinfo) | Cross-platform system info (disk, CPU) |
| [`proc-maps`](https://docs.rs/proc-maps) | Process memory region mapping |

## Architecture

Single-file design (`src/main.rs`) with clear sections:

```
Structs & Parsers  →  Data collection from system APIs
App State          →  Centralized state with per-tab scroll/cursor
UI Functions       →  One render_* function per tab
Main Loop          →  Event handling + 1-second refresh cycle
```

Refresh strategy:
- **Every tick** (~1s): RAM, CPU, VM stats, process maps, I/O, network stats
- **Every 3 ticks**: Activity connections (`lsof`)
- **Every 5 ticks**: Disk list, battery
- **Every 30 ticks**: GPU, cameras, disk hardware

## License

MIT
