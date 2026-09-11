//! Tests for A1 (model), A2 (solver), and the placeholder generator.

use std::time::Instant;

use logicgrid::*;
use rand::prelude::*;

fn e(cat: Cat, item: Item) -> Entity {
    Entity { cat, item }
}

fn cat(name: &str, items: &[&str]) -> Category {
    Category {
        name: name.into(),
        items: items.iter().map(|s| s.to_string()).collect(),
        ..Default::default()
    }
}

/// Hand-written 4×4: Alice Red Cat 2, Bob Green Dog 4, Carol Blue Fish 1, Dave Yellow Bird 3.
fn four_by_four() -> Puzzle {
    let (alice, bob, carol, dave) = (e(0, 0), e(0, 1), e(0, 2), e(0, 3));
    let (red, green, blue, yellow) = (e(1, 0), e(1, 1), e(1, 2), e(1, 3));
    let (cat_, dog, fish) = (e(2, 0), e(2, 1), e(2, 2));
    let floor = |i| e(3, i);
    Puzzle {
        categories: vec![
            cat("Person", &["Alice", "Bob", "Carol", "Dave"]),
            cat("Color", &["Red", "Green", "Blue", "Yellow"]),
            cat("Pet", &["Cat", "Dog", "Fish", "Bird"]),
            cat("Floor", &["1", "2", "3", "4"]),
        ],
        ordered: vec![false, false, false, true],
        clues: vec![
            Clue::Before(carol, alice, 3),
            Clue::Adjacent(alice, dave, 3),
            Clue::Is(bob, floor(3)),
            Clue::Is(red, cat_),
            Clue::Either(alice, red, green),
            Clue::IsNot(alice, green),
            Clue::Is(fish, floor(0)),
            Clue::IsNot(cat_, floor(2)),
            Clue::After(yellow, blue, 3),
            Clue::Is(green, dog),
            Clue::NotSame(dave, dog),
        ],
        solution: Solution(vec![
            vec![0, 1, 2, 3],
            vec![0, 1, 2, 3],
            vec![0, 1, 2, 3],
            vec![1, 3, 0, 2],
        ]),
        seed: 0,
        title: String::new(),
        intro: String::new(),
    }
}

fn solve_p(p: &Puzzle) -> SolveResult {
    solve(&p.categories, &p.ordered, &p.clues)
}

// ---- A1 model ----

#[test]
fn a1_2_solution_bijection_and_pairs() {
    let p = four_by_four();
    assert!(p.solution.is_bijection());
    assert!(p.solution.pairs(e(0, 0), e(1, 0))); // Alice-Red
    assert!(p.solution.pairs(e(1, 0), e(2, 0))); // Red-Cat
    assert!(!p.solution.pairs(e(0, 0), e(3, 0))); // Alice not floor 1
    assert!(p.solution.pairs(e(3, 3), e(0, 1))); // floor 4 - Bob
    assert!(!Solution(vec![vec![0, 1], vec![0, 0]]).is_bijection());
    assert!(!Solution(vec![vec![1, 0], vec![0, 1]]).is_bijection());
    assert!(p.validate().is_ok());
}

#[test]
fn a1_3_clue_text() {
    let p = four_by_four();
    let texts: Vec<String> = p.clues.iter().map(|c| c.text(&p)).collect();
    assert_eq!(texts[0], "Carol's Floor comes before Alice's Floor.");
    assert_eq!(
        texts[1],
        "Alice's Floor and Dave's Floor are next to each other."
    );
    assert_eq!(texts[2], "Bob goes with 4.");
    assert_eq!(texts[4], "Alice goes with either Red or Green.");
    assert_eq!(texts[5], "Alice does not go with Green.");
    assert_eq!(texts[8], "Yellow's Floor comes after Blue's Floor.");
    assert_eq!(texts[10], "Dave is not Dog.");
    assert_eq!(
        Clue::Before(e(3, 0), e(0, 0), 3).text(&p),
        "1 comes before Alice's Floor."
    );
}

#[test]
fn a1_4_json_roundtrip() {
    let p = four_by_four();
    let q = Puzzle::from_json(&p.to_json()).unwrap();
    assert_eq!(q.clues, p.clues);
    assert_eq!(q.solution, p.solution);
    assert_eq!(q.categories, p.categories);
    assert!(Puzzle::from_json("{nope").is_err());
}

#[test]
fn validate_rejects_bad_puzzles() {
    let mut p = four_by_four();
    p.clues.push(Clue::Is(e(0, 0), e(9, 0)));
    assert!(p.validate().is_err());
    let mut p = four_by_four();
    p.clues.push(Clue::Is(e(0, 0), e(3, 0)));
    assert!(p.validate().unwrap_err().contains("false"));
    let mut p = four_by_four();
    p.solution.0[1][0] = 1;
    assert!(p.validate().is_err());
}

#[test]
fn size_and_difficulty_parse() {
    assert_eq!("4x4".parse::<Size>().unwrap(), Size { cats: 4, items: 4 });
    assert_eq!("3X5".parse::<Size>().unwrap(), Size { cats: 3, items: 5 });
    assert!("4".parse::<Size>().is_err());
    assert_eq!("Hard".parse::<Difficulty>().unwrap(), Difficulty::Hard);
    assert!("brutal".parse::<Difficulty>().is_err());
}

// ---- A2 solver ----

#[test]
fn a2_1_unique_multiple_none() {
    let p = four_by_four();
    assert_eq!(solve_p(&p), SolveResult::Unique(p.solution.clone()));

    let mut fewer = p.clone();
    fewer.clues.pop();
    assert_eq!(solve_p(&fewer), SolveResult::Multiple);

    let mut contra = p.clone();
    contra.clues.push(Clue::Is(e(0, 0), e(3, 2)));
    assert_eq!(solve_p(&contra), SolveResult::None);
}

#[test]
fn a2_1_each_clue_type_is_necessary_somewhere() {
    // Each clue in the 4x4 is exercised: dropping any one must not yield None.
    let p = four_by_four();
    for i in 0..p.clues.len() {
        let mut q = p.clone();
        q.clues.remove(i);
        assert_ne!(solve_p(&q), SolveResult::None, "clue {i}");
    }
}

#[test]
fn transitivity_propagates_across_pairs() {
    // Alice=Red, Red=Cat, Cat=1 fully determines Alice's row via chains only.
    let cats = vec![
        cat("P", &["A", "B", "C"]),
        cat("Q", &["r", "g", "b"]),
        cat("R", &["c", "d", "f"]),
        cat("S", &["1", "2", "3"]),
    ];
    let clues = vec![
        Clue::Is(e(0, 0), e(1, 0)),
        Clue::Is(e(1, 0), e(2, 0)),
        Clue::Is(e(2, 0), e(3, 0)),
        Clue::Is(e(0, 1), e(1, 1)),
        Clue::Is(e(1, 1), e(2, 1)),
        Clue::Is(e(2, 1), e(3, 1)),
    ];
    let sol = Solution(vec![vec![0, 1, 2]; 4]);
    assert_eq!(solve(&cats, &[false; 4], &clues), SolveResult::Unique(sol));
}

#[test]
fn ordinal_clues_use_item_order() {
    let cats = vec![cat("P", &["A", "B", "C"]), cat("N", &["1", "2", "3"])];
    let (a, b, c) = (e(0, 0), e(0, 1), e(0, 2));
    // A < B, A adjacent B, B adjacent C  => A=1, B=2, C=3
    let clues = vec![
        Clue::Before(a, b, 1),
        Clue::Adjacent(a, b, 1),
        Clue::Adjacent(b, c, 1),
    ];
    assert_eq!(
        solve(&cats, &[false, true], &clues),
        SolveResult::Unique(Solution(vec![vec![0, 1, 2], vec![0, 1, 2]]))
    );
    // A > C, C < B, A > B => C=1, B=2, A=3
    let clues = vec![
        Clue::After(a, c, 1),
        Clue::Before(c, b, 1),
        Clue::After(a, b, 1),
    ];
    assert_eq!(
        solve(&cats, &[false, true], &clues),
        SolveResult::Unique(Solution(vec![vec![0, 1, 2], vec![2, 1, 0]]))
    );
    // Impossible chain.
    let clues = vec![
        Clue::Before(a, b, 1),
        Clue::Before(b, c, 1),
        Clue::Before(c, a, 1),
    ];
    assert_eq!(solve(&cats, &[false, true], &clues), SolveResult::None);
}

#[test]
fn either_across_categories() {
    let cats = vec![
        cat("P", &["A", "B", "C"]),
        cat("Q", &["r", "g", "b"]),
        cat("R", &["c", "d", "f"]),
    ];
    // A is r or c (not both); A=r is false => A=c. Then also r != c.
    let clues = vec![
        Clue::Either(e(0, 0), e(1, 0), e(2, 0)),
        Clue::IsNot(e(0, 0), e(1, 0)),
        Clue::Is(e(0, 1), e(1, 0)),
        Clue::Is(e(0, 1), e(2, 1)),
        Clue::Is(e(0, 0), e(1, 1)),
    ];
    let sol = Solution(vec![vec![0, 1, 2], vec![1, 0, 2], vec![0, 1, 2]]);
    assert_eq!(solve(&cats, &[false; 3], &clues), SolveResult::Unique(sol));
    // Both true is a contradiction.
    let clues = vec![
        Clue::Either(e(0, 0), e(1, 0), e(2, 0)),
        Clue::Is(e(0, 0), e(1, 0)),
        Clue::Is(e(0, 0), e(2, 0)),
    ];
    assert_eq!(solve(&cats, &[false; 3], &clues), SolveResult::None);
}

#[test]
fn a2_2_five_by_five_few_clues_is_fast() {
    let cats: Vec<Category> = (0..5)
        .map(|c| cat(&format!("C{c}"), &["a", "b", "c", "d", "e"]))
        .collect();
    let clues = vec![
        Clue::Before(e(0, 0), e(0, 1), 4),
        Clue::Adjacent(e(1, 0), e(2, 0), 4),
        Clue::Either(e(0, 2), e(1, 1), e(2, 2)),
        Clue::IsNot(e(0, 3), e(3, 3)),
    ];
    let t = Instant::now();
    assert_eq!(
        solve(&cats, &[false, false, false, false, true], &clues),
        SolveResult::Multiple
    );
    assert_eq!(
        solve(&cats, &[false, false, false, false, true], &[]),
        SolveResult::Multiple
    );
    assert!(t.elapsed().as_millis() < 1000, "{:?}", t.elapsed());
}

/// Random consistent clue sets on 3x3..5x5: the solver must never say None, and when it
/// says Unique it must be the planted solution (soundness + completeness).
#[test]
fn a2_2_random_consistent_clues_never_none() {
    let mut worst = std::time::Duration::ZERO;
    for seed in 0..100u64 {
        let mut rng = StdRng::seed_from_u64(seed);
        let n = rng.random_range(3..=5);
        let m = rng.random_range(3..=5);
        let cats: Vec<Category> = (0..n)
            .map(|c| Category {
                name: format!("C{c}"),
                items: (0..m).map(|i| format!("{c}-{i}")).collect(),
                ..Default::default()
            })
            .collect();
        let mut sol = vec![(0..m).collect::<Vec<_>>()];
        for _ in 1..n {
            let mut row: Vec<usize> = (0..m).collect();
            row.shuffle(&mut rng);
            sol.push(row);
        }
        let sol = Solution(sol);
        let ord = n - 1;
        let re = |rng: &mut StdRng| e(rng.random_range(0..n), rng.random_range(0..m));
        let mut clues = vec![];
        let mut result = SolveResult::Multiple;
        while result == SolveResult::Multiple && clues.len() < 40 {
            let (a, b, c) = (re(&mut rng), re(&mut rng), re(&mut rng));
            let cand = match rng.random_range(0..7) {
                0 => Clue::Is(a, b),
                1 => Clue::IsNot(a, b),
                2 => Clue::Either(a, b, c),
                3 => Clue::Before(a, b, ord),
                4 => Clue::After(a, b, ord),
                5 => Clue::Adjacent(a, b, ord),
                _ => Clue::NotSame(a, b),
            };
            if cand.entities().windows(2).any(|w| w[0].cat == w[1].cat) || !cand.holds(&sol) {
                continue;
            }
            clues.push(cand);
            let t = Instant::now();
            result = solve(&cats, &vec![false; n], &clues);
            worst = worst.max(t.elapsed());
            if let SolveResult::Unique(s) = &result {
                assert_eq!(s, &sol, "seed {seed}");
            }
            assert_ne!(result, SolveResult::None, "seed {seed} clues {clues:?}");
        }
    }
    assert!(worst.as_millis() < 1000, "worst {worst:?}");
}

// ---- A2.3 grid ----

#[test]
fn a2_3_grid_contradictions_and_solved() {
    let p = four_by_four();
    let mut g = Grid::new(&p);
    assert_eq!(g.get(e(0, 0), e(1, 0)), Mark::Unknown);
    g.set(e(0, 0), e(1, 0), Mark::Yes); // Alice-Red: correct
    g.set(e(1, 1), e(0, 0), Mark::No); // Green-Alice: correct, set reversed
    assert_eq!(g.get(e(0, 0), e(1, 1)), Mark::No);
    assert!(g.contradictions(&p).is_empty());
    g.set(e(0, 0), e(3, 0), Mark::Yes); // Alice-floor1: wrong
    g.set(e(2, 2), e(3, 0), Mark::No); // Fish-floor1: wrong
    let bad = g.contradictions(&p);
    assert_eq!(bad.len(), 2);
    assert!(bad.contains(&(e(0, 0), e(3, 0))));
    assert!(bad.contains(&(e(2, 2), e(3, 0))));
    assert!(!g.is_solved(&p));

    let mut g = Grid::new(&p);
    for i in 0..4 {
        for c in 1..4 {
            g.set(e(0, i), e(c, p.solution.0[c][i]), Mark::Yes);
        }
    }
    assert!(g.is_solved(&p));
    g.set(e(1, 0), e(2, 1), Mark::Yes); // Red-Dog wrong
    assert!(!g.is_solved(&p));
}

// ---- placeholder generator ----

#[test]
fn placeholder_generate_is_unique_and_valid() {
    let p = generate(Size { cats: 4, items: 4 }, Difficulty::Hard, 7, None);
    assert_eq!(p.seed, 7);
    assert!(p.validate().is_ok());
    assert_eq!(solve_p(&p), SolveResult::Unique(p.solution.clone()));
    for c in &p.clues {
        assert!(!c.text(&p).is_empty());
    }
}
