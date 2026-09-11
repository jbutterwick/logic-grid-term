//! Puzzle generator (A3): pick a theme, shuffle a solution, enumerate true clues, add until
//! unique, then strip redundant clues so the set is minimal.

use rand::prelude::*;

use crate::themes::{THEMES, Theme};
use crate::{Cat, Category, Clue, Difficulty, Entity, Puzzle, Size, Solution, SolveResult, solve};

/// Names of the built-in themes, indexable by `generate`'s `theme` argument.
pub fn theme_names() -> impl Iterator<Item = &'static str> {
    THEMES.iter().map(|t| t.name)
}

/// Generate a puzzle with exactly one solution and a minimal clue set (A3.1–A3.7).
///
/// Deterministic in `(size, difficulty, seed, theme)`. `theme` indexes [`theme_names`]; `None`
/// picks one from the seed. Supports 3–5 categories of 3–5 items; a size outside the theme
/// bank is clamped to it.
pub fn generate(size: Size, difficulty: Difficulty, seed: u64, theme: Option<usize>) -> Puzzle {
    let n = size.cats.clamp(3, 5);
    let m = size.items.clamp(3, 5);
    let mut rng = StdRng::seed_from_u64(seed);
    let theme = match theme {
        Some(i) => &THEMES[i % THEMES.len()],
        None => THEMES.choose(&mut rng).expect("theme bank is non-empty"),
    };
    let (categories, ordered) = pick_theme(&mut rng, theme, n, m);
    let solution = random_solution(&mut rng, n, m);
    let clues = candidates(&solution, &ordered, difficulty, n, m);
    let unique =
        |cs: &[Clue]| solve(&categories, &ordered, cs) == SolveResult::Unique(solution.clone());

    // Hard must actually use ordinal and Either clues (A3.4). Minimization can strip them, so
    // retry with a fresh shuffle; ~1 in 5 first tries fails on 3x3, almost never on larger sizes.
    // ponytail: bounded retry rather than steering the minimizer; last attempt is accepted as-is.
    let mut chosen = Vec::new();
    for attempt in 0..20 {
        chosen = minimal_unique(&mut rng, clues.clone(), unique);
        let ok = difficulty != Difficulty::Hard
            || attempt == 19
            || (chosen.iter().any(|c| c.ord().is_some())
                && chosen.iter().any(|c| matches!(c, Clue::Either(..))));
        if ok {
            break;
        }
    }
    chosen.shuffle(&mut rng);

    Puzzle {
        categories,
        ordered,
        clues: chosen,
        solution,
        seed,
        title: theme.name.into(),
        intro: theme.intro.into(),
    }
}

/// Shuffle `clues`, add them until `unique`, then drop every clue whose removal keeps
/// uniqueness (A3.3). Every true clue is consistent, so the full set is always unique: it
/// contains all `Is` clues, or for Hard all `Is` clues between non-anchor categories plus
/// ordinal clues that pin each anchor item.
fn minimal_unique(
    rng: &mut StdRng,
    mut clues: Vec<Clue>,
    unique: impl Fn(&[Clue]) -> bool,
) -> Vec<Clue> {
    clues.shuffle(rng);
    let mut chosen: Vec<Clue> = Vec::new();
    for c in clues {
        chosen.push(c);
        if unique(&chosen) {
            break;
        }
    }
    debug_assert!(unique(&chosen), "full candidate set must be unique");
    // Later-added clues are tried first since the early ones tend to carry more information.
    let mut i = chosen.len();
    while i > 0 {
        i -= 1;
        let c = chosen.remove(i);
        if !unique(&chosen) {
            chosen.insert(i, c);
        }
    }
    chosen
}

/// Pick a theme and trim it to `n` categories × `m` items, keeping the anchor and at least one
/// ordered category. Ordered items keep their relative order.
fn pick_theme(rng: &mut StdRng, theme: &Theme, n: usize, m: usize) -> (Vec<Category>, Vec<bool>) {
    let theme = &theme.cats;
    let mut rest: Vec<usize> = (1..theme.len()).collect();
    rest.shuffle(rng);
    rest.truncate(n - 1);
    if !rest.iter().any(|&i| theme[i].ordered) {
        let ord = (1..theme.len())
            .filter(|&i| theme[i].ordered)
            .choose(rng)
            .expect("every theme has an ordered category");
        rest[0] = ord;
    }
    let cats: Vec<usize> = std::iter::once(0).chain(rest).collect();
    let categories = cats
        .iter()
        .map(|&i| {
            let t = &theme[i];
            let mut idx: Vec<usize> = (0..t.items.len()).collect();
            idx.shuffle(rng);
            idx.truncate(m);
            if t.ordered {
                idx.sort_unstable();
            }
            Category {
                name: t.name.into(),
                items: idx.into_iter().map(|j| t.items[j].to_string()).collect(),
                phrase: t.phrase.into(),
                less: t.less.into(),
                more: t.more.into(),
                adjacent: t.adjacent.into(),
            }
        })
        .collect();
    let ordered = cats.iter().map(|&i| theme[i].ordered).collect();
    (categories, ordered)
}

fn random_solution(rng: &mut StdRng, n: usize, m: usize) -> Solution {
    Solution(
        (0..n)
            .map(|c| {
                let mut row: Vec<usize> = (0..m).collect();
                if c > 0 {
                    row.shuffle(rng);
                }
                row
            })
            .collect(),
    )
}

/// Every true clue of the difficulty's allowed types.
fn candidates(sol: &Solution, ordered: &[bool], d: Difficulty, n: usize, m: usize) -> Vec<Clue> {
    let e = |cat: Cat, item| Entity { cat, item };
    let ents: Vec<Entity> = (0..n).flat_map(|c| (0..m).map(move |i| e(c, i))).collect();
    let mut out = Vec::new();
    for &a in &ents {
        for &b in &ents {
            if a.cat >= b.cat {
                continue;
            }
            // A3.7: Hard never states an anchor pairing directly.
            let leaks = d == Difficulty::Hard && a.cat == 0;
            if sol.pairs(a, b) {
                if !leaks {
                    out.push(Clue::Is(a, b));
                }
            } else {
                out.push(Clue::IsNot(a, b));
                if d != Difficulty::Easy {
                    out.push(Clue::NotSame(a, b));
                }
            }
            if d == Difficulty::Easy {
                continue;
            }
            // Either(a, b, c): b and c in the same category, exactly one pairs with a.
            for &c in &ents {
                if c.cat == b.cat && c.item != b.item && sol.pairs(a, b) != sol.pairs(a, c) {
                    out.push(Clue::Either(a, b, c));
                }
            }
            if d != Difficulty::Hard {
                continue;
            }
            for o in (0..n).filter(|&o| ordered[o]) {
                let (pa, pb) = (sol.pos(a, o), sol.pos(b, o));
                if pa < pb {
                    out.push(Clue::Before(a, b, o));
                } else if pa > pb {
                    out.push(Clue::After(a, b, o));
                }
                if pa.abs_diff(pb) == 1 {
                    out.push(Clue::Adjacent(a, b, o));
                }
            }
        }
    }
    out
}
