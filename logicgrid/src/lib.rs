//! Logic grid puzzle model, solver, and generator (no UI dependencies).
//!
//! A puzzle has `N` categories of `M` items each. Category 0 is the anchor;
//! the [`Solution`] maps every anchor item to one item of every other category.

#![deny(missing_docs)]
mod clue;
mod generate;
mod model;
mod solver;
mod themes;

pub use clue::Clue;
pub use generate::generate;
pub use model::{Cat, Category, Difficulty, Entity, Grid, Item, Mark, Puzzle, Size, Solution};
pub use solver::{SolveResult, solve};
