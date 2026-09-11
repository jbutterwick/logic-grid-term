//! Hand-written 4x4 and 5x5 puzzles for layout testing while the generator is a placeholder.

use logicgrid::{Category, Clue, Entity, Puzzle, Solution};

fn cat(name: &str, items: &[&str]) -> Category {
    Category {
        name: name.into(),
        items: items.iter().map(|s| s.to_string()).collect(),
        ..Default::default()
    }
}

const fn e(cat: usize, item: usize) -> Entity {
    Entity { cat, item }
}

/// Person / Pet / Floor(ordered) / Drink.
/// Ann: Dog 2 Tea. Ben: Cat 4 Juice. Cal: Fish 1 Water. Dee: Hamster 3 Coffee.
pub fn four() -> Puzzle {
    let (ann, ben, cal, dee) = (e(0, 0), e(0, 1), e(0, 2), e(0, 3));
    let (cat_, dog, fish, hamster) = (e(1, 0), e(1, 1), e(1, 2), e(1, 3));
    let (f1, f3, f4) = (e(2, 0), e(2, 2), e(2, 3));
    let (tea, coffee, juice, water) = (e(3, 0), e(3, 1), e(3, 2), e(3, 3));
    Puzzle {
        categories: vec![
            cat("Person", &["Ann", "Ben", "Cal", "Dee"]),
            cat("Pet", &["Cat", "Dog", "Fish", "Hamster"]),
            cat("Floor", &["1", "2", "3", "4"]),
            cat("Drink", &["Tea", "Coffee", "Juice", "Water"]),
        ],
        ordered: vec![false, false, true, false],
        clues: vec![
            Clue::Is(ann, dog),
            Clue::After(ben, dee, 2),
            Clue::Before(cal, ann, 2),
            Clue::Is(fish, f1),
            Clue::Is(coffee, f3),
            Clue::IsNot(ben, tea),
            Clue::IsNot(cat_, water),
            Clue::Adjacent(ann, dee, 2),
            Clue::Either(juice, cat_, hamster),
            Clue::IsNot(hamster, f4),
            Clue::Adjacent(ben, dee, 2),
            Clue::Before(water, tea, 2),
        ],
        solution: Solution(vec![
            vec![0, 1, 2, 3],
            vec![1, 0, 2, 3],
            vec![1, 3, 0, 2],
            vec![0, 2, 3, 1],
        ]),
        seed: 4,
        title: String::new(),
        intro: String::new(),
    }
}

/// Person / Pet / Floor(ordered) / Drink / Color.
/// Ann: Dog 2 Tea Blue. Ben: Cat 4 Juice Red. Cal: Fish 1 Water White.
/// Dee: Hamster 3 Coffee Green. Eve: Parrot 5 Milk Black.
pub fn five() -> Puzzle {
    let (ann, ben, cal, dee, eve) = (e(0, 0), e(0, 1), e(0, 2), e(0, 3), e(0, 4));
    let (cat_, dog, fish, hamster, parrot) = (e(1, 0), e(1, 1), e(1, 2), e(1, 3), e(1, 4));
    let (f1, f3, f4, f5) = (e(2, 0), e(2, 2), e(2, 3), e(2, 4));
    let (tea, coffee, juice, water, milk) = (e(3, 0), e(3, 1), e(3, 2), e(3, 3), e(3, 4));
    let (red, blue, green, black, white) = (e(4, 0), e(4, 1), e(4, 2), e(4, 3), e(4, 4));
    Puzzle {
        categories: vec![
            cat("Person", &["Ann", "Ben", "Cal", "Dee", "Eve"]),
            cat("Pet", &["Cat", "Dog", "Fish", "Hamster", "Parrot"]),
            cat("Floor", &["1", "2", "3", "4", "5"]),
            cat("Drink", &["Tea", "Coffee", "Juice", "Water", "Milk"]),
            cat("Color", &["Red", "Blue", "Green", "Black", "White"]),
        ],
        ordered: vec![false, false, true, false, false],
        clues: vec![
            Clue::Is(eve, f5),
            Clue::Before(cal, ann, 2),
            Clue::After(ben, dee, 2),
            Clue::Adjacent(ann, dee, 2),
            Clue::Adjacent(ben, dee, 2),
            Clue::Is(ann, dog),
            Clue::Is(fish, f1),
            Clue::IsNot(hamster, f4),
            Clue::IsNot(hamster, f5),
            Clue::After(parrot, cat_, 2),
            Clue::Is(coffee, f3),
            Clue::IsNot(ben, tea),
            Clue::IsNot(cat_, water),
            Clue::Either(juice, cat_, hamster),
            Clue::Before(water, tea, 2),
            Clue::IsNot(eve, tea),
            Clue::Is(red, juice),
            Clue::Is(white, fish),
            Clue::Either(black, milk, coffee),
            Clue::Adjacent(blue, green, 2),
            Clue::Before(blue, green, 2),
        ],
        solution: Solution(vec![
            vec![0, 1, 2, 3, 4],
            vec![1, 0, 2, 3, 4],
            vec![1, 3, 0, 2, 4],
            vec![0, 2, 3, 1, 4],
            vec![1, 0, 4, 2, 3],
        ]),
        seed: 5,
        title: String::new(),
        intro: String::new(),
    }
}

pub fn fixture(n: usize) -> Result<Puzzle, String> {
    match n {
        4 => Ok(four()),
        5 => Ok(five()),
        _ => Err("--fixture takes 4 or 5".into()),
    }
}

#[cfg(test)]
mod tests {
    use logicgrid::{SolveResult, solve};

    use super::*;

    #[test]
    fn fixtures_are_valid_and_unique() {
        for p in [four(), five()] {
            p.validate().unwrap();
            assert_eq!(
                solve(&p.categories, &p.ordered, &p.clues),
                SolveResult::Unique(p.solution.clone()),
                "{}x{}",
                p.n_cats(),
                p.n_items()
            );
        }
    }
}
