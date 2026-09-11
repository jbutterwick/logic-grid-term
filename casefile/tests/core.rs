use casefile::*;
use logicgrid::{SolveResult, solve};

fn settings(level: Level, seed: u64) -> Settings {
    Settings {
        seed,
        level,
        theme: None,
        adult: false,
    }
}

const LEVELS: [Level; 3] = [Level::Easy, Level::Medium, Level::Hard];

#[test]
fn deterministic() {
    for level in LEVELS {
        let a = Game::new(settings(level, 7)).to_json();
        let b = Game::new(settings(level, 7)).to_json();
        assert_eq!(a, b);
        let g = Game::from_json(&a).unwrap();
        assert_eq!(g.to_json(), a);
    }
}

#[test]
fn cases_are_sound() {
    for level in LEVELS {
        for seed in 1..=5 {
            let c = Case::new(settings(level, seed));
            let p = &c.puzzle;
            let truthful: Vec<Clue> = c
                .suspects
                .iter()
                .flat_map(|s| &s.statements)
                .filter(|st| st.truthful)
                .map(|st| st.clue.clone())
                .collect();
            assert_eq!(truthful.len(), p.clues.len(), "{level:?}/{seed}");
            assert_eq!(
                solve(&p.categories, &p.ordered, &truthful),
                SolveResult::Unique(p.solution.clone()),
                "{level:?}/{seed}"
            );
            for s in &c.suspects {
                for st in s.statements.iter().filter(|st| !st.truthful) {
                    assert!(
                        !st.clue.holds(&p.solution),
                        "{level:?}/{seed} {:?}",
                        st.clue
                    );
                }
                assert_eq!(s.liar, s.statements.iter().any(|st| !st.truthful));
                assert_eq!(
                    s.email,
                    format!("{}@{}", s.name.to_lowercase(), c.frame.domain)
                );
            }
            let liars = c.suspects.iter().filter(|s| s.liar).count();
            assert_eq!(liars, level.liars(), "{level:?}/{seed}");
            assert!(c.guilty.cat > 0);
            assert_eq!(p.solution.0[c.guilty.cat][c.culprit], c.guilty.item);
            if level.culprit_lies() {
                assert!(c.suspects[c.culprit].liar);
                assert!(c.suspects[c.culprit].statements.iter().any(|st| matches!(
                    st.clue,
                    Clue::Is(a, b) if !st.truthful && a.item == c.culprit && b.cat == c.guilty.cat
                )));
            }
            assert!(c.guilty_fact().contains(p.name(c.guilty)));
            assert_eq!(c.noun(), p.categories[0].name.to_lowercase());
        }
    }
}

#[test]
fn first_person_covers_theme_verbs() {
    let pairs = [
        ("owns the Cat", "own the Cat"),
        ("is bound to the Key", "am bound to the Key"),
        ("is 25 years old", "am 25 years old"),
        ("has Locker 101", "have Locker 101"),
        ("flies the Teal flag", "fly the Teal flag"),
        ("was in the Study", "was in the Study"),
        ("arrived at 6:00", "arrived at 6:00"),
        ("claims to have been Reading", "claim to have been Reading"),
        ("always gets the Bagel", "always get the Bagel"),
        ("goes in at 9:00", "go in at 9:00"),
        ("lives on a lower floor than", "live on a lower floor than"),
    ];
    for (third, first) in pairs {
        assert_eq!(first_person(third), first);
    }
}

#[test]
fn render_speaks_grammatically() {
    let bad = ["I is ", "I owns ", "is I.", "is I ", "who I ", "I and "];
    for level in LEVELS {
        for seed in 1..=8 {
            let c = Case::new(settings(level, seed));
            for (i, s) in c.suspects.iter().enumerate() {
                for st in &s.statements {
                    for v in 0..3 {
                        let t = render(&st.clue, &c.puzzle, Some(i), v);
                        for b in bad {
                            assert!(!t.contains(b), "{t:?} contains {b:?}");
                        }
                        assert!(t.ends_with('.'), "{t:?}");
                        assert!(t.chars().next().unwrap().is_uppercase(), "{t:?}");
                        let own = st.clue.entities().iter().any(|e| e.cat == 0 && e.item == i);
                        assert_eq!(
                            own,
                            t.contains("I ") || t.contains("I'm") || t.contains(" me"),
                            "{t:?}"
                        );
                    }
                    let plain = render(&st.clue, &c.puzzle, None, 0);
                    assert!(!plain.contains("I ") && !plain.contains(" me"), "{plain:?}");
                }
            }
        }
    }
}

#[test]
fn ask_delivers_on_next_action() {
    let mut g = Game::new(settings(Level::Easy, 3));
    assert_eq!(g.chief.len(), 1);
    assert_eq!(g.unread(None), 1);
    g.mark_read(None);
    assert_eq!(g.unread(None), 0);
    let qs = g.questions(0);
    assert_eq!(qs[0], Question::Own);
    assert!(!qs.contains(&Question::Press));
    g.ask(0, Question::Own);
    assert_eq!(g.threads[0].len(), 1, "reply must not be immediate");
    assert!(g.threads[0][0].outgoing);
    g.wait();
    assert_eq!(g.threads[0].len(), 2);
    assert_eq!(g.unread(Some(0)), 1);
    assert_eq!(g.threads[0][1].from, g.case.suspects[0].email);
    assert!(g.questions(0).contains(&Question::Press));
    // A second ask also delivers the first reply of the next suspect.
    g.ask(1, Question::Own);
    assert_eq!(g.threads[1].len(), 1);
    g.ask(1, Question::Category(1));
    assert_eq!(g.threads[1].len(), 3);
}

#[test]
fn press_repeats_truth_and_drifts_lies() {
    let mut seen_true = false;
    let mut seen_false = false;
    for seed in 1..=10 {
        let mut g = Game::new(settings(Level::Medium, seed));
        let m = g.case.suspects.len();
        for s in 0..m {
            for q in [Question::Own, Question::Category(1), Question::Category(2)] {
                let flags = |g: &Game| -> Vec<bool> {
                    g.case.suspects[s]
                        .statements
                        .iter()
                        .map(|st| st.revealed)
                        .collect()
                };
                let before = flags(&g);
                g.ask(s, q);
                let Some(idx) = flags(&g).iter().zip(&before).position(|(a, b)| a != b) else {
                    continue;
                };
                g.wait();
                let reply = g.threads[s].last().unwrap().body.clone();
                let st = g.case.suspects[s].statements[idx].clone();
                let stmt = (0..3)
                    .map(|v| render(&st.clue, &g.case.puzzle, Some(s), v))
                    .find(|t| reply.contains(t.as_str()))
                    .expect("reply contains the rendered statement");
                g.ask(s, Question::Press);
                g.wait();
                let pressed = g.threads[s].last().unwrap().body.clone();
                if st.truthful {
                    assert!(
                        pressed.contains(&stmt),
                        "{pressed:?} should repeat {stmt:?}"
                    );
                    seen_true = true;
                } else {
                    assert!(
                        !pressed.contains(&stmt),
                        "{pressed:?} should drift from {stmt:?}"
                    );
                    seen_false = true;
                }
            }
        }
        assert!(g.presses > 0);
    }
    assert!(seen_true && seen_false);
}

#[test]
fn accuse_paths_and_rank() {
    let mut g = Game::new(settings(Level::Medium, 11));
    assert_eq!(g.rank(), Rank::Inspector);
    let culprit = g.case.culprit;
    let innocent = (culprit + 1) % g.case.suspects.len();
    assert!(!g.accuse(innocent, g.case.guilty));
    assert_eq!(g.wrong, 1);
    assert_eq!(g.rank(), Rank::Detective);
    assert!(g.case.suspects[innocent].silenced);
    assert!(g.questions(innocent).is_empty());
    assert!(!g.solved);
    let wrong_fact = Entity {
        cat: g.case.guilty.cat,
        item: (g.case.guilty.item + 1) % g.case.puzzle.n_items(),
    };
    assert!(!g.accuse(culprit, wrong_fact));
    assert_eq!(g.rank(), Rank::Constable);
    assert!(g.accuse(culprit, g.case.guilty));
    assert!(g.solved);
    let last = g.chief.last().unwrap();
    assert!(last.body.contains("Constable"));
    assert!(last.body.contains(&g.case.suspects[culprit].name));
    assert!(last.body.contains(&g.epilogue()));
    assert!(g.epilogue().matches('.').count() >= g.case.suspects.len());
}

#[test]
fn progress_and_nudge() {
    let mut g = Game::new(settings(Level::Easy, 5));
    let (done, total) = g.progress();
    assert_eq!(done, 0);
    assert_eq!(total, g.case.puzzle.clues.len());
    let m = g.case.suspects.len();
    let n = g.case.puzzle.n_cats();
    while g.progress().0 < total {
        let before = g.progress().0;
        for s in 0..m {
            for q in std::iter::once(Question::Own).chain((1..n).map(Question::Category)) {
                g.ask(s, q);
            }
        }
        assert!(
            g.progress().0 > before,
            "every truthful statement must be reachable"
        );
    }
    g.wait();
    assert_eq!(g.progress(), (total, total));
    assert_eq!(g.chief.len(), 2, "nudge arrives once past halfway");
    // Dodge when nothing is left.
    g.ask(0, Question::Own);
    g.wait();
    assert_eq!(g.progress(), (total, total));
}

#[test]
fn level_parsing_and_themes() {
    assert_eq!("HARD".parse::<Level>().unwrap(), Level::Hard);
    assert!("nope".parse::<Level>().is_err());
    assert_eq!(Level::Medium.name(), "medium");
    assert!(theme_names().len() > 5);
    let g = Game::new(Settings {
        seed: 1,
        level: Level::Easy,
        theme: Some(13),
        adult: true,
    });
    assert_eq!(g.case.frame.title, theme_names()[13]);
    assert!(g.chief[0].body.contains(&g.case.guilty_fact()));
}
