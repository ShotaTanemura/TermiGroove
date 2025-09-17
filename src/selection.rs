use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub struct SelectionModel {
    pub items: Vec<PathBuf>,
    pub set: HashSet<PathBuf>,
    pub right_idx: usize,
    pub status: String,
}

impl SelectionModel {
    fn clamp_right_idx(&mut self) {
        if self.items.is_empty() {
            self.right_idx = 0;
        } else if self.right_idx >= self.items.len() {
            self.right_idx = self.items.len() - 1;
        }
    }

    pub fn add_file(&mut self, path: PathBuf) {
        if self.set.insert(path.clone()) {
            self.items.push(path.clone());
            self.right_idx = self.items.len().saturating_sub(1);
            self.status = format!("Added {}", get_file_name(&path));
        } else {
            // toggle remove
            self.remove_file(&path);
        }
    }

    pub fn remove_at_cursor(&mut self) {
        if self.items.is_empty() {
            return;
        }
        let idx = self.right_idx.min(self.items.len() - 1);
        let path = self.items.remove(idx);
        self.set.remove(&path);
        self.status = format!("Removed {}", get_file_name(&path));
        self.clamp_right_idx();
    }

    pub fn remove_file(&mut self, path: &Path) {
        if self.set.remove(path)
            && let Some(pos) = self.items.iter().position(|p| p == path)
        {
            self.items.remove(pos);
            self.status = format!("Removed {}", get_file_name(path));
            self.clamp_right_idx();
        }
    }

    pub fn move_up(&mut self) {
        if self.items.is_empty() {
            return;
        }
        if self.right_idx > 0 {
            self.right_idx -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.items.is_empty() {
            return;
        }
        if self.right_idx + 1 < self.items.len() {
            self.right_idx += 1;
        }
    }
}

fn get_file_name(p: &Path) -> String {
    p.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("?")
        .to_string()
}
