// PLACEHOLDER: replaced by the generator task
//! Puzzle generator (A3). Currently returns one fixed 3×3 puzzle for every input.

use crate::{Category, Clue, Difficulty, Entity, Puzzle, Size, Solution};

/// Generate a puzzle. PLACEHOLDER: ignores `size` and `difficulty`, returns a fixed 3×3 puzzle.
pub fn generate(size: Size, difficulty: Difficulty, seed: u64) -> Puzzle {
    let _ = (size, difficulty);
    let cat = |name: &str, items: &[&str]| Category {
        name: name.into(),
        items: items.iter().map(|s| s.to_string()).collect(),
    };
    let e = |cat, item| Entity { cat, item };
    let (alice, bob, carol) = (e(0, 0), e(0, 1), e(0, 2));
    let (cat_, dog) = (e(1, 0), e(1, 1));
    let three = e(2, 2);
    Puzzle {
        categories: vec![
            cat("Person", &["Alice", "Bob", "Carol"]),
            cat("Pet", &["Cat", "Dog", "Fish"]),
            cat("Floor", &["1", "2", "3"]),
        ],
        ordered: vec![false, false, true],
        clues: vec![
            Clue::Is(alice, cat_),
            Clue::IsNot(carol, dog),
            Clue::Before(bob, alice, 2),
            Clue::Either(carol, three, dog),
        ],
        // Alice: Cat, 2. Bob: Dog, 1. Carol: Fish, 3.
        solution: Solution(vec![vec![0, 1, 2], vec![0, 1, 2], vec![1, 0, 2]]),
        seed,
    }
}
