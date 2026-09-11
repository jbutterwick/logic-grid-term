//! Autosave of in-progress state to the platform data dir (B3.1).

use std::path::{Path, PathBuf};
use std::{fs, io};

use logicgrid::{Grid, Puzzle};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Progress {
    /// Stored alongside progress so the home screen can reopen it (B3.3).
    pub puzzle: Puzzle,
    pub grid: Grid,
    pub struck: Vec<bool>,
    pub solved: bool,
    /// "easy" | "medium" | "hard" | "file" | "fixture"
    pub label: String,
}

pub fn dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("logictui")
}

/// `<seed>-<cats>x<items>-<label>[-<theme>]`
pub fn key(p: &Puzzle, label: &str) -> String {
    let theme: String = p
        .title
        .chars()
        .filter_map(|c| match c {
            ' ' | '-' => Some('-'),
            c if c.is_ascii_alphanumeric() => Some(c.to_ascii_lowercase()),
            _ => None,
        })
        .collect();
    let sep = if theme.is_empty() { "" } else { "-" };
    format!(
        "{}-{}x{}-{}{sep}{theme}",
        p.seed,
        p.n_cats(),
        p.n_items(),
        label
    )
}

pub fn save(dir: &Path, key: &str, progress: &Progress) -> io::Result<()> {
    fs::create_dir_all(dir)?;
    let json = serde_json::to_string(progress).map_err(io::Error::other)?;
    fs::write(dir.join(format!("{key}.json")), json)
}

pub fn load(dir: &Path, key: &str) -> Option<Progress> {
    let text = fs::read_to_string(dir.join(format!("{key}.json"))).ok()?;
    serde_json::from_str(&text).ok()
}

/// All saved progress, newest first.
pub fn list(dir: &Path) -> Vec<(String, Progress)> {
    let Ok(rd) = fs::read_dir(dir) else {
        return vec![];
    };
    let mut entries: Vec<_> = rd
        .flatten()
        .filter_map(|e| {
            let path = e.path();
            let key = path.file_stem()?.to_str()?.to_string();
            let modified = e.metadata().ok()?.modified().ok()?;
            let progress = load(dir, &key)?;
            Some((modified, key, progress))
        })
        .collect();
    entries.sort_by_key(|e| std::cmp::Reverse(e.0));
    entries.into_iter().map(|(_, k, p)| (k, p)).collect()
}

#[cfg(test)]
mod tests {
    use logicgrid::{Entity, Mark};

    use super::*;

    #[test]
    fn round_trip() {
        let dir = std::env::temp_dir().join(format!("logictui-test-{}", std::process::id()));
        let puzzle = crate::fixtures::fixture(4).unwrap();
        let mut grid = Grid::new(&puzzle);
        let (a, b) = (Entity { cat: 0, item: 1 }, Entity { cat: 2, item: 3 });
        grid.set(a, b, Mark::Yes);
        let progress = Progress {
            puzzle: puzzle.clone(),
            grid,
            struck: vec![true, false, true],
            solved: false,
            label: "fixture".into(),
        };
        let k = key(&puzzle, "fixture");
        assert_eq!(k, "4-4x4-fixture");
        save(&dir, &k, &progress).unwrap();
        let back = load(&dir, &k).unwrap();
        assert_eq!(back.grid.get(a, b), Mark::Yes);
        assert_eq!(back.grid.get(a, Entity { cat: 2, item: 0 }), Mark::Unknown);
        assert_eq!(back.struck, vec![true, false, true]);
        assert_eq!(back.puzzle.clues, puzzle.clues);
        assert_eq!(list(&dir)[0].0, k);
        let _ = fs::remove_dir_all(dir);
    }
}
