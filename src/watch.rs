use crate::display::Format;
use procmod_core::Process;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Watchpoint {
    pub label: String,
    pub address: usize,
    pub size: usize,
    pub format: WatchFormat,
    #[serde(skip)]
    pub current: Vec<u8>,
    #[serde(skip)]
    pub previous: Vec<u8>,
    #[serde(skip)]
    pub changed: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum WatchFormat {
    Hex,
    Decimal,
    Float,
    HealthBar,
    Vector3,
    Boolean,
}

impl From<WatchFormat> for Format {
    fn from(wf: WatchFormat) -> Self {
        match wf {
            WatchFormat::Hex => Format::Hex,
            WatchFormat::Decimal => Format::Decimal,
            WatchFormat::Float => Format::Float,
            WatchFormat::HealthBar => Format::HealthBar,
            WatchFormat::Vector3 => Format::Vector3,
            WatchFormat::Boolean => Format::Boolean,
        }
    }
}

impl Watchpoint {
    pub fn new(label: impl Into<String>, address: usize, size: usize, format: WatchFormat) -> Self {
        Self {
            label: label.into(),
            address,
            size,
            format,
            current: vec![0; size],
            previous: vec![0; size],
            changed: false,
        }
    }

    pub fn poll(&mut self, process: &Process) -> bool {
        self.previous.copy_from_slice(&self.current);
        match process.read_bytes(self.address, self.size) {
            Ok(data) => {
                self.current = data;
                self.changed = self.current != self.previous;
                self.changed
            }
            Err(_) => {
                self.changed = false;
                false
            }
        }
    }

    pub fn display_format(&self) -> Format {
        self.format.into()
    }
}

pub struct WatchList {
    pub watches: Vec<Watchpoint>,
}

impl WatchList {
    pub fn new() -> Self {
        Self {
            watches: Vec::new(),
        }
    }

    pub fn add(&mut self, watch: Watchpoint) {
        self.watches.push(watch);
    }

    pub fn remove(&mut self, index: usize) -> Option<Watchpoint> {
        if index < self.watches.len() {
            Some(self.watches.remove(index))
        } else {
            None
        }
    }

    pub fn poll_all(&mut self, process: &Process) -> usize {
        let mut changed_count = 0;
        for w in &mut self.watches {
            if w.poll(process) {
                changed_count += 1;
            }
        }
        changed_count
    }

    pub fn len(&self) -> usize {
        self.watches.len()
    }

    pub fn is_empty(&self) -> bool {
        self.watches.is_empty()
    }
}
