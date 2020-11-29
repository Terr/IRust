use super::IRustError;
use std::fs;
use std::path;

/// Mark to keep backward-compatibility with the old way of saving history
const NEW_HISTORY_MARK: &str = "##NewHistoryMark##\n//\n";

#[derive(Default)]
pub struct History {
    history: Vec<String>,
    cursor: usize,
    history_file_path: path::PathBuf,
    pub lock: bool,
    last_buffer: Vec<char>,
}

impl History {
    pub fn new() -> Result<Self, IRustError> {
        let history_file_path = crate::irust::cargo_cmds::IRUST_DIR.join("history");
        if !history_file_path.exists() {
            fs::File::create(&history_file_path)?;
        }

        let history: String = fs::read_to_string(&history_file_path)?;

        let history: Vec<String> = if history.starts_with(NEW_HISTORY_MARK) {
            history
                .split("\n//\n")
                .skip(1)
                .map(ToOwned::to_owned)
                .collect()
        } else {
            history.lines().map(ToOwned::to_owned).collect()
        };

        let cursor = 0;

        Ok(Self {
            history,
            cursor,
            history_file_path,
            lock: false,
            last_buffer: Vec::new(),
        })
    }
    pub fn down(&mut self, buffer: &Vec<char>) -> Option<String> {
        if !self.lock {
            self.last_buffer = buffer.clone();
            self.cursor = 1;
        }

        self.cursor = self.cursor.saturating_sub(1);
        if self.cursor == 0 {
            return Some(self.last_buffer.iter().copied().collect());
        }

        let (filtered, _filtered_len) = self.filter(&self.last_buffer);

        filtered.map(ToOwned::to_owned)
    }

    pub fn up(&mut self, buffer: &Vec<char>) -> Option<String> {
        if !self.lock {
            self.last_buffer = buffer.clone();
            self.cursor = 0;
        }
        self.cursor += 1;

        let (filtered, filtered_len) = self.filter(&self.last_buffer);
        let res = filtered.map(ToOwned::to_owned);

        if self.cursor + 1 >= filtered_len {
            self.cursor = filtered_len;
        }

        res
    }

    pub fn push(&mut self, buffer: String) {
        if !buffer.is_empty() && Some(&buffer) != self.history.last() {
            self.history.push(buffer);
            self.go_to_last();
        }
    }

    pub fn save(&self) -> Result<(), IRustError> {
        let is_comment = |s: &str| -> bool { s.trim_start().starts_with("//") };
        let mut history = self.history.clone();

        if history.is_empty() || history[0] != NEW_HISTORY_MARK {
            history.insert(0, NEW_HISTORY_MARK.to_string());
        }

        let history: Vec<String> = history
            .into_iter()
            .map(|e| {
                let e: Vec<String> = e
                    .lines()
                    .filter(|l| !is_comment(l))
                    .map(ToOwned::to_owned)
                    .collect();
                e.join("\n")
            })
            .collect();
        let history = history.join("\n//\n");

        fs::write(&self.history_file_path, history)?;
        Ok(())
    }

    fn filter(&self, buffer: &Vec<char>) -> (Option<&String>, usize) {
        let mut f: Vec<&String> = self
            .history
            .iter()
            .filter(|h| h.contains(&buffer.iter().collect::<String>()))
            .rev()
            .collect();
        f.dedup();

        let len = f.len();
        (
            f.get(self.cursor.saturating_sub(1)).map(ToOwned::to_owned),
            len,
        )
    }

    fn go_to_last(&mut self) {
        if !self.history.is_empty() {
            self.cursor = 0;
        }
    }

    pub fn find(&self, needle: &str) -> Option<&String> {
        self.history.iter().find(|h| h.contains(needle))
    }

    pub fn lock(&mut self) {
        self.lock = true;
    }

    pub fn unlock(&mut self) {
        self.lock = false;
    }
}
