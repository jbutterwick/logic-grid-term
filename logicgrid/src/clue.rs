//! Clue types and their English rendering.

use serde::{Deserialize, Serialize};

use crate::{Cat, Entity, Puzzle, Solution};

/// A single puzzle clue (A1.3).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Clue {
    /// `a` is paired with `b`.
    Is(Entity, Entity),
    /// `a` is not paired with `b`.
    IsNot(Entity, Entity),
    /// `a` is paired with exactly one of `b`, `c`.
    Either(Entity, Entity, Entity),
    /// In ordered category `ord`, `a`'s value is strictly less than `b`'s.
    Before(Entity, Entity, Cat),
    /// In ordered category `ord`, `a`'s value is strictly greater than `b`'s.
    After(Entity, Entity, Cat),
    /// In ordered category `ord`, `a` and `b` differ by exactly one position.
    Adjacent(Entity, Entity, Cat),
    /// Two items from different categories are not the same entity.
    NotSame(Entity, Entity),
}

impl Clue {
    /// Entities mentioned by the clue, in order.
    pub fn entities(&self) -> Vec<Entity> {
        match *self {
            Clue::Is(a, b)
            | Clue::IsNot(a, b)
            | Clue::NotSame(a, b)
            | Clue::Before(a, b, _)
            | Clue::After(a, b, _)
            | Clue::Adjacent(a, b, _) => vec![a, b],
            Clue::Either(a, b, c) => vec![a, b, c],
        }
    }

    /// The ordered category the clue refers to, if any.
    pub fn ord(&self) -> Option<Cat> {
        match *self {
            Clue::Before(_, _, o) | Clue::After(_, _, o) | Clue::Adjacent(_, _, o) => Some(o),
            _ => None,
        }
    }

    /// True if every referenced category/item index fits an `n`×`m` puzzle.
    pub fn in_range(&self, n: usize, m: usize) -> bool {
        self.entities().iter().all(|e| e.cat < n && e.item < m) && self.ord().is_none_or(|o| o < n)
    }

    /// True if the clue is satisfied by `sol`.
    pub fn holds(&self, sol: &Solution) -> bool {
        match *self {
            Clue::Is(a, b) => sol.pairs(a, b),
            Clue::IsNot(a, b) | Clue::NotSame(a, b) => !sol.pairs(a, b),
            Clue::Either(a, b, c) => sol.pairs(a, b) != sol.pairs(a, c),
            Clue::Before(a, b, o) => sol.pos(a, o) < sol.pos(b, o),
            Clue::After(a, b, o) => sol.pos(a, o) > sol.pos(b, o),
            Clue::Adjacent(a, b, o) => sol.pos(a, o).abs_diff(sol.pos(b, o)) == 1,
        }
    }

    /// English rendering using the puzzle's display names.
    pub fn text(&self, p: &Puzzle) -> String {
        let n = |e: Entity| p.name(e);
        // "3" if e is itself in the ordered category, else "Alice's Floor".
        let v = |e: Entity, o: Cat| {
            if e.cat == o {
                n(e).to_string()
            } else {
                format!("{}'s {}", n(e), p.categories[o].name)
            }
        };
        match *self {
            Clue::Is(a, b) => format!("{} goes with {}.", n(a), n(b)),
            Clue::IsNot(a, b) => format!("{} does not go with {}.", n(a), n(b)),
            Clue::Either(a, b, c) => format!("{} goes with either {} or {}.", n(a), n(b), n(c)),
            Clue::Before(a, b, o) => format!("{} comes before {}.", v(a, o), v(b, o)),
            Clue::After(a, b, o) => format!("{} comes after {}.", v(a, o), v(b, o)),
            Clue::Adjacent(a, b, o) => {
                format!("{} and {} are next to each other.", v(a, o), v(b, o))
            }
            Clue::NotSame(a, b) => format!("{} is not {}.", n(a), n(b)),
        }
    }
}
