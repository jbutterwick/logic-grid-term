//! Clue prose, optionally in a suspect's first person.

use logicgrid::{Cat, Clue, Entity, Puzzle};

/// Third-person-singular verb phrase to first person: "owns the Cat" -> "own the Cat",
/// "is bound to" -> "am bound to", "has" -> "have", "flies" -> "fly", "was" -> "was".
pub fn first_person(phrase: &str) -> String {
    let mut words: Vec<String> = phrase.split(' ').map(str::to_string).collect();
    // ponytail: skip a leading adverb ("always gets") and conjugate the next word.
    let i = usize::from(matches!(
        words.first().map(String::as_str),
        Some("always" | "never")
    ));
    let Some(v) = words.get(i) else {
        return phrase.to_string();
    };
    let fp = match v.as_str() {
        "is" => "am".to_string(),
        "was" => "was".to_string(),
        "has" => "have".to_string(),
        "does" => "do".to_string(),
        "goes" => "go".to_string(),
        w if w.len() > 4 && w.ends_with("ies") => format!("{}y", &w[..w.len() - 3]),
        w if ["sses", "shes", "ches", "xes", "zes"]
            .iter()
            .any(|s| w.ends_with(s)) =>
        {
            w[..w.len() - 2].to_string()
        }
        w if w.ends_with('s') && !w.ends_with("ss") => w[..w.len() - 1].to_string(),
        w => w.to_string(),
    };
    words[i] = fp;
    words.join(" ")
}

/// Render a clue as prose. `speaker = Some(i)` renders in first person for suspect i
/// ("I was in the Study", "I'm not the guest who gave the Wine", "Crane and I sat next
/// to each other"). `variant` picks among 2-3 wordings; any usize is valid (mod).
pub fn render(clue: &Clue, p: &Puzzle, speaker: Option<usize>, variant: usize) -> String {
    let noun = p.categories[0].name.to_lowercase();
    let me = |e: Entity| e.cat == 0 && speaker == Some(e.item);
    // "owns the Cat"; for the anchor "is Crane" / "is me".
    let does = |e: Entity| {
        if me(e) {
            return "is me".to_string();
        }
        let ph = &p.categories[e.cat].phrase;
        let ph = if ph.is_empty() { "goes with {}" } else { ph };
        ph.replace("{}", p.name(e))
    };
    // Subject form: "I" / "Crane" / "the guest who owns the Cat".
    let who = |e: Entity| match (me(e), e.cat) {
        (true, _) => "I".to_string(),
        (false, 0) => p.name(e).to_string(),
        _ => format!("the {noun} who {}", does(e)),
    };
    // Object form: "me" instead of "I".
    let obj = |e: Entity| if me(e) { "me".to_string() } else { who(e) };
    // "I own the Cat" / "Crane owns the Cat".
    let says = |s: Entity, pred: String| {
        if me(s) {
            format!("I {}", first_person(&pred))
        } else {
            format!("{} {pred}", who(s))
        }
    };
    let ord = |o: Cat| &p.categories[o];
    // Anchor entity first so "I"/"Crane" leads; symmetric clues only.
    let lead = |a: Entity, b: Entity| {
        if b.cat == 0 && a.cat != 0 {
            (b, a)
        } else {
            (a, b)
        }
    };

    let s = match *clue {
        Clue::Is(a, b) => {
            let (a, b) = lead(a, b);
            match variant % 3 {
                0 => format!("{}.", says(a, does(b))),
                1 if a.cat == 0 => format!("It was {} who {}.", obj(a), does(b)),
                1 => format!("Whoever {} also {}.", does(a), does(b)),
                _ => format!("{} {}.", who(b), does(a)),
            }
        }
        Clue::IsNot(a, b) | Clue::NotSame(a, b) => {
            let (a, b) = lead(a, b);
            match variant % 3 {
                0 if me(a) => format!("I'm not {}.", who(b)),
                0 => format!("{} is not {}.", who(a), who(b)),
                1 => format!("{} is someone other than {}.", who(b), obj(a)),
                _ if me(a) => format!("{} and I are not the same {noun}.", who(b)),
                _ => format!("{} and {} are not the same {noun}.", who(a), who(b)),
            }
        }
        Clue::Either(a, b, c) => match variant % 2 {
            0 if me(a) => format!(
                "I either {} or {}.",
                first_person(&does(b)),
                first_person(&does(c))
            ),
            0 => format!("{} either {} or {}.", who(a), does(b), does(c)),
            _ => format!("Either {} or {} {}.", who(b), who(c), does(a)),
        },
        Clue::Before(a, b, o) | Clue::After(b, a, o) => match variant % 2 {
            0 => format!("{} {}.", says(a, ord(o).less.clone()), obj(b)),
            _ => format!("{} {}.", says(b, ord(o).more.clone()), obj(a)),
        },
        Clue::Adjacent(a, b, o) => {
            // "Crane and I", never "I and Crane".
            let (a, b) = if me(a) { (b, a) } else { (a, b) };
            match variant % 3 {
                0 => format!("{} and {} {}.", who(a), who(b), ord(o).adjacent),
                1 => format!(
                    "{} and {} {}, as it happens.",
                    who(a),
                    who(b),
                    ord(o).adjacent
                ),
                _ => format!(
                    "It's true that {} and {} {}.",
                    who(a),
                    who(b),
                    ord(o).adjacent
                ),
            }
        }
    };
    let mut c = s.chars();
    c.next()
        .map_or(s.clone(), |f| f.to_uppercase().chain(c).collect())
}
