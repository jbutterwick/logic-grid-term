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

    /// English rendering. Themed wording when the puzzle carries flavor phrases, else plain.
    pub fn text(&self, p: &Puzzle) -> String {
        if p.categories.iter().any(|c| c.phrase.is_empty()) {
            return self.plain(p);
        }
        let noun = p.categories[0].name.to_lowercase();
        // "owns the Cat" / for the anchor, "is Alice".
        let does = |e: Entity| p.categories[e.cat].phrase.replace("{}", p.name(e));
        // "Alice" / "the person who owns the Cat".
        let who = |e: Entity| {
            if e.cat == 0 {
                p.name(e).to_string()
            } else {
                format!("the {noun} who {}", does(e))
            }
        };
        let ord = |o: Cat| &p.categories[o];
        // ponytail: variant chosen by entity indices, not the seed; deterministic and cheap.
        let v = self
            .entities()
            .iter()
            .map(|e| e.cat * 7 + e.item * 3)
            .sum::<usize>();
        let s = match *self {
            Clue::Is(a, b) => match v % 3 {
                0 => format!("{} {}.", who(a), does(b)),
                1 => format!("{} {}.", who(b), does(a)),
                _ if a.cat == 0 => format!("it is {} who {}.", who(a), does(b)),
                _ => format!("whoever {} also {}.", does(a), does(b)),
            },
            Clue::IsNot(a, b) | Clue::NotSame(a, b) => match v % 3 {
                0 => format!("{} is not {}.", who(a), who(b)),
                1 => format!("{} is someone other than {}.", who(b), who(a)),
                _ => format!("{} and {} are not the same {noun}.", who(a), who(b)),
            },
            Clue::Either(a, b, c) => match v % 2 {
                0 => format!("{} either {} or {}.", who(a), does(b), does(c)),
                _ => format!("either {} or {} {}.", who(b), who(c), does(a)),
            },
            Clue::Before(a, b, o) | Clue::After(b, a, o) => match v % 2 {
                0 => format!("{} {} {}.", who(a), ord(o).less, who(b)),
                _ => format!("{} {} {}.", who(b), ord(o).more, who(a)),
            },
            Clue::Adjacent(a, b, o) => match v % 2 {
                0 => format!("{} and {} {}.", who(a), who(b), ord(o).adjacent),
                _ => format!("{} and {} {}.", who(b), who(a), ord(o).adjacent),
            },
        };
        let mut c = s.chars();
        c.next()
            .map_or(s.clone(), |f| f.to_uppercase().chain(c).collect())
    }

    /// Plain rendering using only display names.
    fn plain(&self, p: &Puzzle) -> String {
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
