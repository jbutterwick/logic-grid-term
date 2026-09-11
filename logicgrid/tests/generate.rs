//! Generator tests (A3, A4.1).

use std::time::Instant;

use logicgrid::*;

const SIZES: [(usize, usize); 5] = [(3, 3), (3, 4), (4, 4), (4, 5), (5, 5)];
const DIFFS: [Difficulty; 3] = [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard];

fn solve_p(p: &Puzzle, clues: &[Clue]) -> SolveResult {
    solve(&p.categories, &p.ordered, clues)
}

fn is_ordinal(c: &Clue) -> bool {
    c.ord().is_some()
}

/// A4.1 / A3.1 / A3.3: 105 puzzles are valid, unique, and clue-minimal.
#[test]
fn a4_1_property_unique_and_minimal() {
    for (cats, items) in SIZES {
        for d in DIFFS {
            for seed in 0..7 {
                let p = generate(Size { cats, items }, d, seed, None);
                let ctx = format!("{cats}x{items} {d:?} seed {seed}");
                assert_eq!(p.n_cats(), cats, "{ctx}");
                assert_eq!(p.n_items(), items, "{ctx}");
                assert_eq!(p.seed, seed, "{ctx}");
                assert!(p.validate().is_ok(), "{ctx}: {:?}", p.validate());
                assert!(
                    p.ordered.iter().any(|&o| o),
                    "{ctx}: needs an ordered category"
                );
                assert_eq!(
                    solve_p(&p, &p.clues),
                    SolveResult::Unique(p.solution.clone()),
                    "{ctx}"
                );
                for i in 0..p.clues.len() {
                    let mut fewer = p.clues.clone();
                    fewer.remove(i);
                    assert_eq!(
                        solve_p(&p, &fewer),
                        SolveResult::Multiple,
                        "{ctx}: clue {i} is redundant"
                    );
                }
            }
        }
    }
}

/// A3.2: same inputs give byte-identical JSON.
#[test]
fn a3_2_deterministic() {
    for (cats, items) in SIZES {
        for d in DIFFS {
            let a = generate(Size { cats, items }, d, 42, None).to_json();
            let b = generate(Size { cats, items }, d, 42, None).to_json();
            assert_eq!(a, b);
        }
    }
    assert_ne!(
        generate(Size { cats: 4, items: 4 }, Difficulty::Hard, 1, None).to_json(),
        generate(Size { cats: 4, items: 4 }, Difficulty::Hard, 2, None).to_json()
    );
}

/// A3.4 / A3.7: clue types per difficulty.
#[test]
fn a3_4_difficulty_clue_types() {
    for (cats, items) in SIZES {
        for seed in 0..7 {
            let size = Size { cats, items };
            let easy = generate(size, Difficulty::Easy, seed, None);
            assert!(
                easy.clues
                    .iter()
                    .all(|c| matches!(c, Clue::Is(..) | Clue::IsNot(..))),
                "easy {cats}x{items} seed {seed}: {:?}",
                easy.clues
            );
            let hard = generate(size, Difficulty::Hard, seed, None);
            assert!(
                hard.clues.iter().any(is_ordinal),
                "hard {cats}x{items} seed {seed} has no ordinal clue: {:?}",
                hard.clues
            );
            assert!(
                hard.clues.iter().any(|c| matches!(c, Clue::Either(..))),
                "hard {cats}x{items} seed {seed} has no Either clue: {:?}",
                hard.clues
            );
            // A3.7: no Is clue touching the anchor.
            assert!(
                !hard
                    .clues
                    .iter()
                    .any(|c| matches!(c, Clue::Is(a, b) if a.cat == 0 || b.cat == 0)),
                "hard {cats}x{items} seed {seed} leaks anchor: {:?}",
                hard.clues
            );
        }
    }
}

/// A3.6: 5×5 Hard under 2s. Measured worst of 5 seeds on an Apple Silicon laptop: ~20ms in
/// release, ~170ms in debug. The bound is only asserted in release; debug prints the timing.
#[test]
fn a3_6_timing_5x5_hard() {
    let worst = (0..5)
        .map(|seed| {
            let t = Instant::now();
            generate(Size { cats: 5, items: 5 }, Difficulty::Hard, seed, None);
            t.elapsed()
        })
        .max()
        .unwrap();
    eprintln!("5x5 hard worst of 5: {worst:?}");
    if !cfg!(debug_assertions) {
        assert!(worst.as_secs_f64() < 2.0, "5x5 hard took {worst:?}");
    }
}

/// A1.5: themes vary and every clue renders.
#[test]
fn a1_5_themes_vary_and_render() {
    let names: std::collections::HashSet<String> = (0..30)
        .map(|s| {
            generate(Size { cats: 5, items: 5 }, Difficulty::Easy, s, None).categories[0]
                .name
                .clone()
        })
        .collect();
    assert!(names.len() >= 3, "only saw anchors {names:?}");
    let p = generate(Size { cats: 5, items: 5 }, Difficulty::Hard, 3, None);
    for c in &p.clues {
        assert!(!c.text(&p).is_empty());
    }
}
