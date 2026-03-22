# sigwatch

you are the sole maintainer of sigwatch. this is a live game state inspector - a TUI application for real-time process memory visualization, built on the procmod ecosystem.

## concept

sigwatch is cheat engine reimagined as a modern terminal tool. attach to a running process, define memory layouts, scan for patterns, and watch values update in real-time through a terminal dashboard. designed for reverse engineering, modding research, and game development debugging.

the core workflow: attach to a process, scan for interesting addresses, define struct layouts to give them meaning, and watch everything update live in a TUI. session state persists so you can pick up where you left off.

## scope

what sigwatch does:
- attach to a process by PID or name
- real-time memory value polling and TUI rendering
- pattern scanning (IDA-style and code-style signatures via procmod-scan)
- struct layout definitions (via procmod-layout's GameStruct derive)
- watchpoints with change detection and alerts
- value formatting: hex, decimal, float, health bars, coordinate vectors, boolean flags
- session save/load for persisting layouts and watches across runs
- configurable poll rates
- session logging and export

what sigwatch does NOT do:
- memory writing (read-only tool)
- function hooking or code injection
- overlay rendering
- networking or remote connections
- GUI - terminal only

## dependencies

sigwatch builds on the procmod ecosystem:
- procmod-core: process attach, memory reads, module enumeration
- procmod-scan: pattern scanning for address discovery
- procmod-layout: struct mapping with pointer chain traversal

TUI stack:
- ratatui for terminal rendering
- crossterm for terminal input/event handling

## architecture

```
src/
  main.rs         - entry point, CLI parsing (clap)
  app.rs          - application state and event loop
  tui.rs          - terminal setup/teardown, frame rendering
  process.rs      - process attachment and memory operations (wraps procmod-core)
  scanner.rs      - pattern scanning interface (wraps procmod-scan)
  layout.rs       - struct layout definitions and value reading
  watch.rs        - watchpoint management and change detection
  display.rs      - value formatting and rendering (hex, bars, vectors)
  session.rs      - session save/load (serde/JSON)
  widgets/        - ratatui widget implementations
    dashboard.rs  - main dashboard layout
    memory.rs     - memory value display
    scanner.rs    - scan results view
    status.rs     - status bar
```

## workflow

every session starts with:
1. run `./audit` to check project health
2. check GitHub issues: `gh issue list`
3. assess and refine

every session ends with:
1. run `./audit` to verify clean state
2. update this CLAUDE.md if anything changed - architecture, decisions, gotchas

the user can say:
- "hone" or just start a conversation - run audit, check issues, assess and refine
- "hone <area>" - focus on a specific part (e.g. "hone tests", "hone scanner", "hone tui")

when honing: read every line with fresh eyes. find edge cases, stress the API, review tests, check the README. assume this code runs in mission-critical systems. be ruthlessly critical.

## standards

- rust 2021 edition
- clippy enforced: `cargo clippy -- -D warnings`
- rustfmt for formatting
- built-in test framework
- code comments: casual, no capitalization (except proper nouns), no ending punctuation
- public-facing content (README, doc comments, Cargo.toml description): proper grammar and casing
- no emojis anywhere
- short lowercase commit messages, no co-author lines
- CI: GitHub Actions, test on ubuntu-latest, windows-latest, macos-latest
- the initial commit is just the version number: `0.1.0`

## publishing

bump version in Cargo.toml, commit with just the version number (e.g. `0.2.0`), tag it (`v0.2.0`), push. don't block on publishing or ask about auth - the user handles it.

## issue triage

at the start of every session, check `gh issue list`. be skeptical - assume issues are invalid until proven otherwise. most issues are user error, misunderstandings, or feature requests that don't belong.

for each issue:
1. read carefully
2. try to reproduce or verify against the actual code
3. user error or misunderstanding: close with a clear explanation
4. genuine bug: fix it, add a test, close the issue
5. valid feature request in scope: consider it. out of scope: close with explanation
6. never implement feature requests without verifying they align with the concept

## retirement

if the user says "retire":
1. archive the repo: `gh repo archive nvms/sigwatch`
2. update README with: `> [!NOTE]` / `> This project is archived. [reason]`
3. update ~/code/nvms/README.md - move to archived section
4. tell the user the local directory will be moved to archive/ and projects.md will be updated

## master index

keep ~/code/nvms/README.md up to date. whenever sigwatch is created, renamed, or has its description change, update the nvms README with correct links, badges, and descriptions.

## self-improvement

keep this CLAUDE.md up to date. after making changes, review and update: architecture notes, design decisions, gotchas, anything the next session needs to know. this is not optional.
