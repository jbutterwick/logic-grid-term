//! Puzzle data model: categories, entities, solutions, and the player grid.

use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::Clue;

/// A named category with its display items (e.g. "Pet": Cat, Dog, Fish).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Category {
    /// Display name of the category.
    pub name: String,
    /// Display names of the items; for ordered categories, index order is the ordinal order.
    pub items: Vec<String>,
}

/// Index into `Puzzle::categories`.
pub type Cat = usize;
/// Index into `Category::items`.
pub type Item = usize;

/// One item of one category.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct Entity {
    /// Category index.
    pub cat: Cat,
    /// Item index within the category.
    pub item: Item,
}

/// `solution[cat][anchor_item] = item` index in `cat`. `solution[0][i] == i`.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Solution(pub Vec<Vec<Item>>);

impl Solution {
    /// The anchor item that entity `e` belongs to.
    pub fn anchor_of(&self, e: Entity) -> Item {
        self.0[e.cat]
            .iter()
            .position(|&x| x == e.item)
            .expect("solution rows are permutations")
    }

    /// True if `a` and `b` belong to the same anchor item.
    pub fn pairs(&self, a: Entity, b: Entity) -> bool {
        self.anchor_of(a) == self.anchor_of(b)
    }

    /// Ordinal position of `e` in category `ord` (its item index there).
    pub fn pos(&self, e: Entity, ord: Cat) -> Item {
        self.0[ord][self.anchor_of(e)]
    }

    /// True if row 0 is the identity and every row is a permutation of `0..m` (A1.2).
    pub fn is_bijection(&self) -> bool {
        let m = self.0.first().map_or(0, |r| r.len());
        m > 0
            && self.0[0].iter().enumerate().all(|(i, &x)| i == x)
            && self.0.iter().all(|row| {
                let mut seen = vec![false; m];
                row.len() == m
                    && row
                        .iter()
                        .all(|&x| x < m && !std::mem::replace(&mut seen[x], true))
            })
    }
}

/// A complete puzzle: categories, clues, and the intended solution.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Puzzle {
    /// `categories[0]` is the anchor.
    pub categories: Vec<Category>,
    /// `ordered[c] == true` if category `c` is numeric/ordinal (item index order).
    pub ordered: Vec<bool>,
    /// The clues shown to the player.
    pub clues: Vec<Clue>,
    /// The unique solution.
    pub solution: Solution,
    /// Seed the puzzle was generated from.
    pub seed: u64,
}

impl Puzzle {
    /// Number of categories.
    pub fn n_cats(&self) -> usize {
        self.categories.len()
    }

    /// Number of items per category.
    pub fn n_items(&self) -> usize {
        self.categories.first().map_or(0, |c| c.items.len())
    }

    /// Display name of an entity.
    pub fn name(&self, e: Entity) -> &str {
        &self.categories[e.cat].items[e.item]
    }

    /// Pretty-printed JSON (A1.4).
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("puzzle serializes")
    }

    /// Parse a puzzle from JSON. Call [`Puzzle::validate`] afterwards on untrusted input.
    pub fn from_json(s: &str) -> Result<Puzzle, serde_json::Error> {
        serde_json::from_str(s)
    }

    /// Structural check: shapes, bijective solution, in-range clues, and clues true under the
    /// solution.
    pub fn validate(&self) -> Result<(), String> {
        let (n, m) = (self.n_cats(), self.n_items());
        if n < 2 || m < 2 {
            return Err("need at least 2 categories and 2 items".into());
        }
        if self.categories.iter().any(|c| c.items.len() != m) {
            return Err("all categories must have the same number of items".into());
        }
        if self.ordered.len() != n {
            return Err("ordered must have one flag per category".into());
        }
        if self.solution.0.len() != n || !self.solution.is_bijection() {
            return Err("solution is not a bijection".into());
        }
        for (i, c) in self.clues.iter().enumerate() {
            if !c.in_range(n, m) {
                return Err(format!("clue {i} references an out-of-range entity"));
            }
            if !c.holds(&self.solution) {
                return Err(format!("clue {i} is false under the solution"));
            }
        }
        Ok(())
    }
}

/// A player's mark on one grid cell.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
pub enum Mark {
    /// Not yet decided.
    #[default]
    Unknown,
    /// The two entities are not paired.
    No,
    /// The two entities are paired.
    Yes,
}

/// Player grid: `mark(a, b)` for entities in different categories, symmetric.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Grid {
    n: usize,
    m: usize,
    marks: Vec<Mark>,
}

impl Grid {
    /// Empty grid sized for `p`.
    pub fn new(p: &Puzzle) -> Grid {
        let (n, m) = (p.n_cats(), p.n_items());
        Grid {
            n,
            m,
            marks: vec![Mark::Unknown; (n * m) * (n * m)],
        }
    }

    fn idx(&self, a: Entity, b: Entity) -> usize {
        (a.cat * self.m + a.item) * self.n * self.m + b.cat * self.m + b.item
    }

    /// Mark between `a` and `b`; `Unknown` if they share a category.
    pub fn get(&self, a: Entity, b: Entity) -> Mark {
        if a.cat == b.cat {
            return Mark::Unknown;
        }
        self.marks[self.idx(a, b)]
    }

    /// Set the mark between `a` and `b` (both directions). No-op if they share a category.
    pub fn set(&mut self, a: Entity, b: Entity, m: Mark) {
        if a.cat == b.cat {
            return;
        }
        let (i, j) = (self.idx(a, b), self.idx(b, a));
        self.marks[i] = m;
        self.marks[j] = m;
    }

    /// All entities in different categories, each unordered pair once.
    fn pairs(&self) -> impl Iterator<Item = (Entity, Entity)> + '_ {
        let (n, m) = (self.n, self.m);
        (0..n).flat_map(move |ca| {
            (ca + 1..n).flat_map(move |cb| {
                (0..m).flat_map(move |ia| {
                    (0..m)
                        .map(move |ib| (Entity { cat: ca, item: ia }, Entity { cat: cb, item: ib }))
                })
            })
        })
    }

    /// Entities whose current mark contradicts `p.solution` (A2.3).
    pub fn contradictions(&self, p: &Puzzle) -> Vec<(Entity, Entity)> {
        self.pairs()
            .filter(|&(a, b)| match self.get(a, b) {
                Mark::Unknown => false,
                Mark::Yes => !p.solution.pairs(a, b),
                Mark::No => p.solution.pairs(a, b),
            })
            .collect()
    }

    /// Every anchor item has its solution `Yes` in every other category and nothing contradicts
    /// (B2.6).
    pub fn is_solved(&self, p: &Puzzle) -> bool {
        let anchor_done = (0..self.m).all(|i| {
            (1..self.n).all(|c| {
                let a = Entity { cat: 0, item: i };
                let b = Entity {
                    cat: c,
                    item: p.solution.0[c][i],
                };
                self.get(a, b) == Mark::Yes
            })
        });
        anchor_done && self.contradictions(p).is_empty()
    }
}

/// Puzzle difficulty level.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Difficulty {
    /// Direct clues only.
    Easy,
    /// Mixed clue types.
    Medium,
    /// Ordinal and Either clues required.
    Hard,
}

impl FromStr for Difficulty {
    type Err = String;

    /// Parses `"easy"`, `"medium"`, or `"hard"` (case-insensitive).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "easy" => Ok(Difficulty::Easy),
            "medium" => Ok(Difficulty::Medium),
            "hard" => Ok(Difficulty::Hard),
            _ => Err(format!("unknown difficulty '{s}' (easy|medium|hard)")),
        }
    }
}

/// Puzzle dimensions: categories × items.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Size {
    /// Number of categories.
    pub cats: usize,
    /// Number of items per category.
    pub items: usize,
}

impl FromStr for Size {
    type Err = String;

    /// Parses `"4x4"` as `cats = 4, items = 4`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err = || format!("bad size '{s}' (expected CATSxITEMS, e.g. 4x4)");
        let (c, i) = s.split_once(['x', 'X']).ok_or_else(err)?;
        let cats = c.trim().parse().map_err(|_| err())?;
        let items = i.trim().parse().map_err(|_| err())?;
        Ok(Size { cats, items })
    }
}
