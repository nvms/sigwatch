<p align="center">
  <img src="https://raw.githubusercontent.com/nvms/sigwatch/main/logo.svg" width="128" height="128" alt="sigwatch" style="border-radius: 16px;" />
</p>

# sigwatch

Live game state inspector. A TUI application for real-time process memory visualization, built on [procmod](https://github.com/procmod).

Attach to a running process, define memory layouts, and watch values update in real-time. Pattern scan for addresses, set watchpoints on value changes, and visualize game state through a terminal dashboard.

> This project is an experiment in AI-maintained open source - autonomously built, tested, and refined by AI with human oversight. Regular audits, thorough test coverage, continuous refinement. The emphasis is on high quality, rigorously tested, production-grade code.

## Features

- Attach to any process by PID or name
- Define struct layouts with procmod-layout's `#[derive(GameStruct)]` syntax
- Real-time polling and rendering of memory values
- Pattern scanning to discover addresses (IDA-style and code-style signatures)
- Watchpoints that alert on value changes
- Color-coded value display (health bars, coordinates, flags)
- Session logging and export
- Configurable poll rates and display formats

## Usage

```
sigwatch attach <pid>
sigwatch attach --name "game.exe"
```

### Interactive commands

- `scan <signature>` - scan process memory for a pattern
- `watch <address> <type>` - add a value to the dashboard
- `layout <file>` - load a struct layout definition
- `export <file>` - export session log

## Built on procmod

sigwatch uses the [procmod](https://github.com/procmod) ecosystem:

- [procmod-core](https://github.com/procmod/procmod-core) - process memory read/write
- [procmod-scan](https://github.com/procmod/procmod-scan) - pattern scanning
- [procmod-layout](https://github.com/procmod/procmod-layout) - struct mapping with pointer chains

## Installation

```
cargo install sigwatch
```

## License

MIT
