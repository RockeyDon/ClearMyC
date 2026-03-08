# ClearMyC
English | [中文](README_zh.md)

My C drive is full again. I (with LLM) know who to blame — I just need one click to delete them.

A lightweight, standalone Windows 10 desktop app for discovering and cleaning up disk-hogging directories. Powered by an optional LLM integration that automatically identifies safe-to-delete folders (caches, temp files, build artifacts, old logs, etc.).

All configuration is stored locally in `./clear_my_c.json`.

## Features

**Manual Cleanup**
- Add directories you want to monitor and clean
- View real-time folder sizes at a glance (auto-scanned on launch)
- Select/deselect folders with checkboxes
- Move to Recycle Bin (recoverable) or Permanently Delete
- Confirmation dialogs for both delete modes

**LLM-Powered Auto Discovery**
- Connect any OpenAI-compatible LLM API (local or remote)
- BFS scans your drive, filters by configurable strategy rules, then asks the LLM which directories are safe to delete
- Discovered folders are added automatically — LLM-recommended ones pre-selected, others kept unchecked for your review

**Strategy Configuration**
- **Skip List**: semicolon-separated paths to never touch (e.g. `C:\Windows;C:\Program Files`)
- **Min Size**: ignore directories smaller than this threshold
- **Skip Recent**: ignore directories modified within N days

**Multi Language Support**
- switch language at **Settings** (⚙️) → **Theme** tab

**Other**
- Light / Dark / System theme
- API key encrypted (AES-128-CBC) in the config file
- Portable single binary — just drop and run

## Getting Started

### Download

Go to the [Releases](https://github.com/RockeyDon/ClearMyC/releases) page, download the latest `clear_my_c.exe`, and run it directly — no installation required.

### Build from Source

If you prefer to build it yourself:

```bash
# Prerequisites: Windows 10 + Rust toolchain (MSVC target)
cargo build --release
./target/release/clear_my_c.exe
```

### LLM Setup (Optional)

1. Open **Settings** (⚙️) → **Strategy** tab
2. Fill in **BaseUrl** (e.g. `http://localhost:11434/v1`), **Model**, and **ApiKey**
3. Click **Connect** to verify
4. Adjust strategy filters (Skip List, Min Size, Skip Recent)
5. Go to **Folders** tab → click **Auto Discovery**

## Tech Stack

- **GUI**: [Iced](https://github.com/iced-rs/iced) 0.14
- **Async**: Tokio
- **HTTP**: Reqwest (for LLM API calls)
- **Encryption**: AES-128-CBC via `aes` + `cbc` crates
- **Windows API**: `windows` crate for Recycle Bin (`SHFileOperationW`)

## Testing

```bash
cargo test
```

unit tests covering data persistence, encryption round-trips, config state management, UI message handling, LLM edge cases, and more.

## License

MIT
