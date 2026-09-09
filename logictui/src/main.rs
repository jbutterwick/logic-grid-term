use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::Parser;
use logicgrid::{Difficulty, Puzzle, Size};

mod app;
mod fixtures;
mod screens;
mod storage;
mod theme;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Cli {
    /// A saved puzzle JSON to play. Omit to use the menu or --size/--difficulty.
    file: Option<PathBuf>,
    /// Puzzle size as CATSxITEMS (3x3, 3x4, 4x4, 4x5, 5x5).
    #[arg(long)]
    size: Option<Size>,
    /// easy | medium | hard
    #[arg(long)]
    difficulty: Option<Difficulty>,
    /// Generator seed; random if omitted.
    #[arg(long)]
    seed: Option<u64>,
    /// Write the generated/loaded puzzle JSON here and exit.
    #[arg(long)]
    save: Option<PathBuf>,
    /// Dev: play a hand-written fixture (4 or 5).
    #[arg(long, hide = true)]
    fixture: Option<usize>,
}

fn random_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(1)
}

/// Resolve CLI args to a puzzle plus its progress-file label, if the menu is skipped.
fn pick(cli: &Cli) -> Result<Option<(Puzzle, String)>, String> {
    if let Some(path) = &cli.file {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let p = Puzzle::from_json(&text).map_err(|e| e.to_string())?;
        p.validate()?;
        return Ok(Some((p, "file".into())));
    }
    if let Some(n) = cli.fixture {
        return Ok(Some((fixtures::fixture(n)?, "fixture".into())));
    }
    if cli.size.is_none() && cli.difficulty.is_none() {
        return Ok(None);
    }
    let size = cli.size.unwrap_or(Size { cats: 4, items: 4 });
    let diff = cli.difficulty.unwrap_or(Difficulty::Medium);
    let puzzle = logicgrid::generate(size, diff, cli.seed.unwrap_or_else(random_seed));
    Ok(Some((puzzle, app::diff_name(diff))))
}

fn main() {
    let cli = Cli::parse();
    let picked = pick(&cli).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1);
    });
    if let Some(path) = &cli.save {
        let Some((p, _)) = &picked else {
            eprintln!("--save needs a puzzle: pass FILE or --size/--difficulty");
            std::process::exit(1);
        };
        if let Err(e) = std::fs::write(path, p.to_json()) {
            eprintln!("{}: {e}", path.display());
            std::process::exit(1);
        }
        println!("wrote {}", path.display());
        return;
    }
    let app = match picked {
        Some((p, label)) => app::App::with_puzzle(p, label),
        None => app::App::new(),
    };
    // ratatui::init installs a panic hook that restores the terminal (B4.1).
    let mut terminal = ratatui::init();
    let result = app.run(&mut terminal);
    ratatui::restore();
    if let Err(e) = result {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_size_and_difficulty() {
        let cli = Cli::try_parse_from([
            "logictui",
            "--size",
            "4x5",
            "--difficulty",
            "Hard",
            "--seed",
            "7",
        ])
        .unwrap();
        assert_eq!(cli.size, Some(Size { cats: 4, items: 5 }));
        assert_eq!(cli.difficulty, Some(Difficulty::Hard));
        assert_eq!(cli.seed, Some(7));
        assert!(Cli::try_parse_from(["logictui", "--size", "big"]).is_err());
        assert!(Cli::try_parse_from(["logictui", "--difficulty", "brutal"]).is_err());
    }

    #[test]
    fn no_args_opens_menu() {
        let cli = Cli::try_parse_from(["logictui"]).unwrap();
        assert!(pick(&cli).unwrap().is_none());
    }
}
