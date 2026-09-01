use std::fs::{create_dir_all, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

pub struct HistoryManager {
    entries: Vec<String>,
    cursor: Option<usize>,
    saved_draft: Option<String>,
    max_entries: usize,
    persistent: bool,
}

impl HistoryManager {
    pub fn new(max_entries: usize) -> Self {
        let mut mgr = Self {
            entries: Vec::new(),
            cursor: None,
            saved_draft: None,
            max_entries,
            persistent: true,
        };
        mgr.load();
        mgr
    }

    #[allow(dead_code)]
    pub fn new_in_memory(max_entries: usize) -> Self {
        Self {
            entries: Vec::new(),
            cursor: None,
            saved_draft: None,
            max_entries,
            persistent: false,
        }
    }

    pub fn get_history_file_path() -> Option<PathBuf> {
        dirs::config_dir().map(|mut p| {
            p.push("xedis");
            p.push("history");
            p
        })
    }

    pub fn load(&mut self) {
        if let Some(path) = Self::get_history_file_path() {
            if path.exists() {
                if let Ok(file) = std::fs::File::open(&path) {
                    let reader = BufReader::new(file);
                    for line in reader.lines().map_while(Result::ok) {
                        let trimmed = line.trim();
                        if !trimmed.is_empty() {
                            self.entries.push(trimmed.to_string());
                        }
                    }
                }
            }
        }
    }

    pub fn push(&mut self, cmd: &str) {
        let trimmed = cmd.trim();
        if trimmed.is_empty() {
            return;
        }

        // Avoid duplicate consecutive entries
        if self.entries.last().map(|s| s.as_str()) != Some(trimmed) {
            self.entries.push(trimmed.to_string());
            if self.entries.len() > self.max_entries {
                self.entries.remove(0);
            }
            if self.persistent {
                self.append_to_disk(trimmed);
            }
        }

        self.cursor = None;
        self.saved_draft = None;
    }

    fn append_to_disk(&self, cmd: &str) {
        if let Some(path) = Self::get_history_file_path() {
            if let Some(parent) = path.parent() {
                let _ = create_dir_all(parent);
            }
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
                let _ = writeln!(file, "{}", cmd);
            }
        }
    }

    pub fn navigate_prev(&mut self, current_input: &str) -> Option<String> {
        if self.entries.is_empty() {
            return None;
        }

        match self.cursor {
            None => {
                self.saved_draft = Some(current_input.to_string());
                let new_cursor = self.entries.len().saturating_sub(1);
                self.cursor = Some(new_cursor);
                self.entries.get(new_cursor).cloned()
            }
            Some(idx) => {
                if idx > 0 {
                    let new_cursor = idx - 1;
                    self.cursor = Some(new_cursor);
                    self.entries.get(new_cursor).cloned()
                } else {
                    self.entries.first().cloned()
                }
            }
        }
    }

    pub fn navigate_next(&mut self) -> Option<String> {
        match self.cursor {
            None => None,
            Some(idx) => {
                if idx + 1 < self.entries.len() {
                    let new_cursor = idx + 1;
                    self.cursor = Some(new_cursor);
                    self.entries.get(new_cursor).cloned()
                } else {
                    self.cursor = None;
                    self.saved_draft.take()
                }
            }
        }
    }

    pub fn reset_nav(&mut self) {
        self.cursor = None;
        self.saved_draft = None;
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for HistoryManager {
    fn default() -> Self {
        Self::new(500)
    }
}
