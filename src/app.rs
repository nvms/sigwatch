use crate::layout;
use crate::scanner::{self, ScanSession};
use crate::session::Session;
use crate::watch::{WatchFormat, WatchList, Watchpoint};
use procmod_core::Process;
use ratatui::widgets::{ListState, TableState};
use std::time::Instant;

pub enum InputMode {
    Normal,
    Command,
}

pub enum Panel {
    Watches,
    Scanner,
    Modules,
}

pub struct App {
    pub process: Process,
    pub watch_list: WatchList,
    pub scan: Option<ScanSession>,
    pub input_mode: InputMode,
    pub input_buffer: String,
    pub active_panel: Panel,
    pub selected_index: usize,
    pub watch_table_state: TableState,
    pub scanner_list_state: ListState,
    pub poll_rate_ms: u64,
    pub status_message: Option<(String, Instant)>,
    pub running: bool,
}

impl App {
    pub fn new(process: Process, session: Option<Session>) -> Self {
        let mut watch_list = WatchList::new();
        let poll_rate_ms;

        if let Some(s) = session {
            for w in s.watches {
                watch_list.add(w);
            }
            poll_rate_ms = s.poll_rate_ms;
        } else {
            poll_rate_ms = 100;
        }

        Self {
            process,
            watch_list,
            scan: None,
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            active_panel: Panel::Watches,
            selected_index: 0,
            watch_table_state: TableState::default().with_selected(Some(0)),
            scanner_list_state: ListState::default().with_selected(Some(0)),
            poll_rate_ms,
            status_message: None,
            running: true,
        }
    }

    fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some((msg.into(), Instant::now()));
    }

    pub fn active_status(&self) -> Option<&str> {
        self.status_message.as_ref().and_then(|(msg, when)| {
            if when.elapsed().as_secs() < 5 {
                Some(msg.as_str())
            } else {
                None
            }
        })
    }

    pub fn poll(&mut self) {
        self.watch_list.poll_all(&self.process);
        if let Some(s) = &mut self.scan {
            s.refresh(&self.process);
        }
    }

    pub fn scanner_len(&self) -> usize {
        self.scan.as_ref().map_or(0, |s| s.candidates.len())
    }

    pub fn execute_command(&mut self, input: &str) {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        match parts[0] {
            "scan" => self.cmd_scan(&parts[1..]),
            "narrow" => self.cmd_narrow(&parts[1..]),
            "reset" => self.cmd_reset(),
            "watch" => self.cmd_watch(&parts[1..]),
            "pick" => self.cmd_pick(&parts[1..]),
            "layout" => self.cmd_layout(&parts[1..]),
            "export" => self.cmd_export(&parts[1..]),
            "rate" => self.cmd_rate(&parts[1..]),
            "remove" | "rm" => self.cmd_remove(&parts[1..]),
            "save" => self.cmd_save(&parts[1..]),
            "help" | "?" => self.cmd_help(),
            "quit" | "q" => self.running = false,
            _ => self.set_status(format!("unknown command: {}", parts[0])),
        }
    }

    fn cmd_scan(&mut self, args: &[&str]) {
        if args.is_empty() {
            self.set_status("usage: scan <value>  (e.g. scan 75.0, scan 42, scan DE AD)");
            return;
        }
        let input = args.join(" ");
        let parsed = match scanner::parse_value(&input) {
            Ok(p) => p,
            Err(e) => {
                self.set_status(format!("bad value: {e}"));
                return;
            }
        };
        match scanner::initial_scan(&self.process, &parsed) {
            Ok(session) => {
                let count = session.candidates.len();
                let vtype = session.value_type.label();
                self.scan = Some(session);
                self.set_status(format!(
                    "{count} candidates ({vtype}) - change the value, then :narrow <new-value>"
                ));
                self.active_panel = Panel::Scanner;
                self.selected_index = 0;
                self.sync_selection();
            }
            Err(e) => self.set_status(format!("scan failed: {e}")),
        }
    }

    fn cmd_narrow(&mut self, args: &[&str]) {
        if args.is_empty() {
            self.set_status("usage: narrow <new-value>  (e.g. narrow 50.0)");
            return;
        }
        let session = match &mut self.scan {
            Some(s) => s,
            None => {
                self.set_status("no active scan - run :scan first");
                return;
            }
        };
        let input = args.join(" ");
        let parsed = match scanner::parse_value(&input) {
            Ok(p) => p,
            Err(e) => {
                self.set_status(format!("bad value: {e}"));
                return;
            }
        };
        let removed = scanner::narrow(&self.process, session, &parsed);
        let remaining = session.candidates.len();
        if remaining == 0 {
            self.set_status("0 candidates left - all filtered out. :reset to start over");
        } else if remaining <= 5 {
            self.set_status(format!(
                "{remaining} candidates left (-{removed}) - press enter to watch one"
            ));
        } else {
            self.set_status(format!(
                "{remaining} candidates left (-{removed}) - keep narrowing"
            ));
        }
        self.selected_index = 0;
        self.sync_selection();
    }

    fn cmd_reset(&mut self) {
        self.scan = None;
        self.selected_index = 0;
        self.sync_selection();
        self.set_status("scan cleared");
    }

    fn cmd_watch(&mut self, args: &[&str]) {
        if args.len() < 2 {
            self.set_status("usage: watch <address> <type> [label]");
            return;
        }

        let address = match parse_address(args[0]) {
            Some(a) => a,
            None => {
                self.set_status(format!("invalid address: {}", args[0]));
                return;
            }
        };

        let (format, size) = match parse_type(args[1]) {
            Some(fs) => fs,
            None => {
                self.set_status(format!("unknown type: {}", args[1]));
                return;
            }
        };

        let label = if args.len() > 2 {
            args[2..].join(" ")
        } else {
            format!("0x{address:X}")
        };

        self.watch_list
            .add(Watchpoint::new(label, address, size, format));
        self.set_status(format!("watching 0x{address:X}"));
    }

    fn cmd_pick(&mut self, args: &[&str]) {
        let session = match &self.scan {
            Some(s) => s,
            None => {
                self.set_status("no scan results");
                return;
            }
        };

        if session.candidates.is_empty() {
            self.set_status("no candidates");
            return;
        }

        let idx = if args.is_empty() {
            self.selected_index
        } else {
            match args[0].parse::<usize>() {
                Ok(i) if i < session.candidates.len() => i,
                _ => {
                    self.set_status(format!(
                        "invalid index (0-{})",
                        session.candidates.len() - 1
                    ));
                    return;
                }
            }
        };

        let addr = session.candidates[idx].address;
        let (wfmt, size) = scan_type_to_watch(session.value_type);
        let label = if args.len() > 1 {
            args[1..].join(" ")
        } else {
            format!("0x{addr:X}")
        };

        self.watch_list
            .add(Watchpoint::new(label, addr, size, wfmt));
        self.set_status(format!("watching 0x{addr:X}"));
        self.active_panel = Panel::Watches;
        self.selected_index = self.watch_list.len().saturating_sub(1);
        self.sync_selection();
    }

    fn cmd_layout(&mut self, args: &[&str]) {
        if args.len() < 2 {
            self.set_status("usage: layout <file> <base-address>");
            return;
        }

        let base = match parse_address(args[1]) {
            Some(a) => a,
            None => {
                self.set_status(format!("invalid address: {}", args[1]));
                return;
            }
        };

        match layout::load_layout(args[0]) {
            Ok(def) => {
                let count = def.fields.len();
                for w in def.to_watchpoints(base) {
                    self.watch_list.add(w);
                }
                self.set_status(format!("loaded {count} fields"));
            }
            Err(e) => self.set_status(format!("layout error: {e}")),
        }
    }

    fn cmd_export(&mut self, args: &[&str]) {
        if args.is_empty() {
            self.set_status("usage: export <file>");
            return;
        }
        let session = self.to_session();
        match crate::session::save(&session, args[0]) {
            Ok(_) => self.set_status(format!("exported to {}", args[0])),
            Err(e) => self.set_status(format!("export failed: {e}")),
        }
    }

    fn cmd_rate(&mut self, args: &[&str]) {
        if args.is_empty() {
            self.set_status(format!("poll rate: {}ms", self.poll_rate_ms));
            return;
        }
        match args[0].parse::<u64>() {
            Ok(ms) if ms >= 10 => {
                self.poll_rate_ms = ms;
                self.set_status(format!("poll rate set to {ms}ms"));
            }
            _ => self.set_status("rate must be >= 10ms"),
        }
    }

    fn cmd_remove(&mut self, args: &[&str]) {
        if args.is_empty() {
            self.set_status("usage: remove <index>");
            return;
        }
        match args[0].parse::<usize>() {
            Ok(i) => match self.watch_list.remove(i) {
                Some(w) => self.set_status(format!("removed: {}", w.label)),
                None => self.set_status("invalid index"),
            },
            Err(_) => self.set_status("usage: remove <index>"),
        }
    }

    fn cmd_save(&mut self, args: &[&str]) {
        let path = if args.is_empty() {
            match crate::session::auto_save_path(self.process.pid()) {
                Some(p) => {
                    if let Err(e) = crate::session::ensure_session_dir() {
                        self.set_status(format!("save failed: {e}"));
                        return;
                    }
                    p.to_string_lossy().to_string()
                }
                None => {
                    self.set_status("no default save path");
                    return;
                }
            }
        } else {
            args[0].to_string()
        };

        let session = self.to_session();
        match crate::session::save(&session, &path) {
            Ok(_) => self.set_status(format!("saved to {path}")),
            Err(e) => self.set_status(format!("save failed: {e}")),
        }
    }

    fn cmd_help(&mut self) {
        self.set_status(
            "scan <val> | narrow <val> | reset | watch | pick | layout | save | rate | rm | quit",
        );
    }

    fn to_session(&self) -> Session {
        Session {
            pid: self.process.pid(),
            watches: self.watch_list.watches.clone(),
            layouts: Vec::new(),
            poll_rate_ms: self.poll_rate_ms,
        }
    }

    fn sync_selection(&mut self) {
        match self.active_panel {
            Panel::Watches => self.watch_table_state.select(Some(self.selected_index)),
            Panel::Scanner => self.scanner_list_state.select(Some(self.selected_index)),
            Panel::Modules => {}
        }
    }

    pub fn select_next(&mut self) {
        let len = match self.active_panel {
            Panel::Watches => self.watch_list.len(),
            Panel::Scanner => self.scanner_len(),
            Panel::Modules => 0,
        };
        if len > 0 {
            self.selected_index = (self.selected_index + 1).min(len - 1);
        }
        self.sync_selection();
    }

    pub fn select_prev(&mut self) {
        self.selected_index = self.selected_index.saturating_sub(1);
        self.sync_selection();
    }

    pub fn cycle_panel(&mut self) {
        self.active_panel = match self.active_panel {
            Panel::Watches => Panel::Scanner,
            Panel::Scanner => Panel::Modules,
            Panel::Modules => Panel::Watches,
        };
        self.selected_index = 0;
        self.sync_selection();
    }
}

fn scan_type_to_watch(vt: scanner::ValueType) -> (WatchFormat, usize) {
    match vt {
        scanner::ValueType::I16 => (WatchFormat::Decimal, 2),
        scanner::ValueType::I32 => (WatchFormat::Decimal, 4),
        scanner::ValueType::I64 => (WatchFormat::Decimal, 8),
        scanner::ValueType::F32 => (WatchFormat::Float, 4),
        scanner::ValueType::F64 => (WatchFormat::Float, 8),
        scanner::ValueType::Bytes => (WatchFormat::Hex, 1),
    }
}

fn parse_address(s: &str) -> Option<usize> {
    let s = s
        .strip_prefix("0x")
        .or_else(|| s.strip_prefix("0X"))
        .unwrap_or(s);
    usize::from_str_radix(s, 16).ok()
}

fn parse_type(s: &str) -> Option<(WatchFormat, usize)> {
    match s {
        "u8" | "i8" | "byte" => Some((WatchFormat::Hex, 1)),
        "u16" | "i16" | "short" => Some((WatchFormat::Decimal, 2)),
        "u32" | "i32" | "int" => Some((WatchFormat::Decimal, 4)),
        "u64" | "i64" | "long" => Some((WatchFormat::Decimal, 8)),
        "f32" | "float" => Some((WatchFormat::Float, 4)),
        "f64" | "double" => Some((WatchFormat::Float, 8)),
        "bool" => Some((WatchFormat::Boolean, 1)),
        "vec3" => Some((WatchFormat::Vector3, 12)),
        "health" => Some((WatchFormat::HealthBar, 8)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_address() {
        assert_eq!(parse_address("0x1000"), Some(0x1000));
        assert_eq!(parse_address("0X1000"), Some(0x1000));
        assert_eq!(parse_address("1000"), Some(0x1000));
        assert_eq!(parse_address("DEADBEEF"), Some(0xDEADBEEF));
        assert_eq!(parse_address("xyz"), None);
    }

    #[test]
    fn test_parse_type() {
        assert_eq!(parse_type("f32"), Some((WatchFormat::Float, 4)));
        assert_eq!(parse_type("u32"), Some((WatchFormat::Decimal, 4)));
        assert_eq!(parse_type("bool"), Some((WatchFormat::Boolean, 1)));
        assert_eq!(parse_type("vec3"), Some((WatchFormat::Vector3, 12)));
        assert_eq!(parse_type("health"), Some((WatchFormat::HealthBar, 8)));
        assert_eq!(parse_type("garbage"), None);
    }
}
