use casefile::*;

const MOODS: [Mood; 4] = [
    Mood::Cooperative,
    Mood::Nervous,
    Mood::Hostile,
    Mood::Chatty,
];

/// Placeholders the core fills, per template kind.
const CHIEF_KEYS: &[&str] = &[
    "{setting}",
    "{crime}",
    "{names}",
    "{fact}",
    "{noun}",
    "{rank}",
    "{epilogue}",
    "{culprit}",
    "{chief}",
];

/// Sexually explicit terms and slurs that must not appear in any mode (whole-word match).
const DENY: &[&str] = &[
    "cock", "dick", "pussy", "tits", "cunt", "cum", "orgasm", "penis", "vagina", "nude", "naked",
    "porn", "fuck", "fucking", "fucker", "nigger", "faggot", "retard", "tranny", "spic", "chink",
];

fn words(s: &str) -> Vec<String> {
    s.split(|c: char| !c.is_alphanumeric())
        .filter(|w| !w.is_empty())
        .map(str::to_lowercase)
        .collect()
}

/// Any `{...}` token not in `allowed` is a placeholder the core won't fill.
fn unknown_placeholder(s: &str, allowed: &[&str]) -> Option<String> {
    let mut rest = s;
    while let Some(i) = rest.find('{') {
        let tail = &rest[i..];
        let j = tail.find('}')?;
        let tok = &tail[..=j];
        if !allowed.contains(&tok) {
            return Some(tok.to_string());
        }
        rest = &tail[j + 1..];
    }
    None
}

fn every_string() -> Vec<(String, &'static str)> {
    let mut out = Vec::new();
    for t in theme_names().into_iter().chain([""]) {
        let f = frame(t);
        for s in [
            f.setting,
            f.setting_adult,
            f.crime,
            f.crime_adult,
            f.chief,
            f.domain,
        ] {
            out.push((format!("frame {t:?}"), s));
        }
    }
    for m in MOODS {
        let v = voice(m);
        for list in [
            v.openers,
            v.openers_adult,
            v.closers,
            v.closers_adult,
            v.dodges,
            v.dodges_adult,
            v.press_true,
            v.press_false,
            v.press_false_subtle,
            v.conscience,
            v.conscience_adult,
        ] {
            out.extend(list.iter().map(|s| (format!("voice {m:?}"), *s)));
        }
    }
    for adult in [false, true] {
        let c = chief(adult);
        for s in [c.intro, c.nudge, c.wrong, c.solved] {
            out.push((format!("chief adult={adult}"), s));
        }
    }
    out
}

#[test]
fn every_theme_has_its_own_frame() {
    let generic = frame("no such theme");
    assert!(generic.theme.is_empty());
    for t in theme_names() {
        let f = frame(t);
        assert_eq!(f.theme, t, "theme {t:?} fell through to the generic frame");
        assert!(!f.domain.is_empty() && !f.chief.is_empty());
        for s in [f.setting, f.setting_adult] {
            let sentences = s.matches(['.', '!', '?']).count();
            assert!(
                (2..=5).contains(&sentences),
                "{t:?} setting has {sentences} sentences"
            );
        }
        assert_ne!(
            f.setting, f.setting_adult,
            "{t:?} adult setting is unchanged"
        );
        for c in [f.crime, f.crime_adult] {
            assert!(
                c.starts_with("the "),
                "{t:?} crime {c:?} is not a noun phrase"
            );
            assert!(!c.ends_with('.'));
        }
    }
}

#[test]
fn settings_never_name_a_category_or_item() {
    for (i, t) in theme_names().into_iter().enumerate() {
        let case = Case::new(Settings {
            seed: 1,
            level: Level::Hard,
            theme: Some(i),
            adult: false,
        });
        let f = frame(t);
        let names: Vec<String> = case
            .puzzle
            .categories
            .iter()
            .flat_map(|c| std::iter::once(&c.name).chain(&c.items))
            .map(|n| n.to_lowercase())
            .collect();
        for s in [f.setting, f.setting_adult] {
            let lower = s.to_lowercase();
            let ws = words(s);
            for n in &names {
                let hit = if n.contains(' ') {
                    lower.contains(n.as_str())
                } else {
                    ws.iter().any(|w| w == n)
                };
                assert!(!hit, "{t:?} setting names {n:?}: {s}");
            }
        }
    }
}

#[test]
fn voices_meet_minimums_and_adult_lists_match() {
    for m in MOODS {
        let v = voice(m);
        assert_eq!(v.mood, m);
        let min = |name: &str, list: &[&str], n: usize| {
            assert!(list.len() >= n, "{m:?} {name} has {} < {n}", list.len());
            for s in list {
                assert!(!s.trim().is_empty(), "{m:?} {name} has an empty entry");
            }
        };
        min("openers", v.openers, 5);
        min("closers", v.closers, 4);
        min("dodges", v.dodges, 4);
        min("press_true", v.press_true, 3);
        min("press_false", v.press_false, 3);
        min("press_false_subtle", v.press_false_subtle, 3);
        min("conscience", v.conscience, 3);
        assert_eq!(v.openers.len(), v.openers_adult.len(), "{m:?} openers");
        assert_eq!(v.closers.len(), v.closers_adult.len(), "{m:?} closers");
        assert_eq!(v.dodges.len(), v.dodges_adult.len(), "{m:?} dodges");
        assert_eq!(
            v.conscience.len(),
            v.conscience_adult.len(),
            "{m:?} conscience"
        );
        for list in [v.press_true, v.press_false, v.press_false_subtle] {
            for s in list {
                assert_eq!(s.matches("{stmt}").count(), 1, "{m:?} press line {s:?}");
            }
        }
        for s in v.press_false_subtle {
            let hedges = ["i believe", "as far as i recall", "i think"];
            let lower = s.to_lowercase();
            let n = hedges.iter().filter(|h| lower.contains(*h)).count();
            assert!(n <= 1, "{m:?} subtle line {s:?} hedges {n} times");
        }
    }
}

#[test]
fn only_known_placeholders() {
    for (src, s) in every_string() {
        let allowed: &[&str] = if src.starts_with("chief") {
            CHIEF_KEYS
        } else if src.starts_with("voice") {
            &["{stmt}"]
        } else {
            &[]
        };
        assert_eq!(unknown_placeholder(s, allowed), None, "{src}: {s:?}");
    }
    for adult in [false, true] {
        let c = chief(adult);
        for k in ["{setting}", "{crime}", "{names}", "{fact}", "{noun}"] {
            assert!(c.intro.contains(k), "adult={adult} intro lacks {k}");
        }
        for k in ["{rank}", "{culprit}", "{epilogue}"] {
            assert!(c.solved.contains(k), "adult={adult} solved lacks {k}");
        }
        assert!(c.wrong.contains("{noun}"));
        assert!(
            c.nudge.len() < c.intro.len() / 2,
            "adult={adult} nudge is not short"
        );
        assert_ne!(c.intro, chief(!adult).intro);
    }
}

#[test]
fn denylist_absent_everywhere() {
    for (src, s) in every_string() {
        for w in words(s) {
            assert!(!DENY.contains(&w.as_str()), "{src} contains {w:?}: {s:?}");
        }
    }
}

#[test]
fn chief_intro_names_every_suspect_and_the_fact() {
    for (i, t) in theme_names().into_iter().enumerate() {
        for adult in [false, true] {
            let g = Game::new(Settings {
                seed: 3,
                level: Level::Medium,
                theme: Some(i),
                adult,
            });
            let intro = &g.chief[0].body;
            assert_eq!(g.case.frame.title, t);
            for s in &g.case.suspects {
                assert!(
                    intro.contains(&s.name),
                    "{t:?} adult={adult} intro lacks {}",
                    s.name
                );
            }
            let fact = g.case.guilty_fact();
            assert!(
                intro.contains(&fact),
                "{t:?} adult={adult} intro lacks {fact:?}"
            );
            assert!(intro.contains(&g.case.frame.setting));
            assert!(intro.contains(&g.case.frame.crime));
            assert!(
                !intro.contains('{'),
                "{t:?} adult={adult} unfilled placeholder: {intro}"
            );
        }
    }
}
