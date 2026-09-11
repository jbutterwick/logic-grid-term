//! Case generation: suspects, traits, and statements from a `logicgrid` puzzle (D1.2-D1.5).

use std::str::FromStr;

use logicgrid::{Clue, Difficulty, Entity, Puzzle, Size, generate};
use rand::prelude::*;
use serde::{Deserialize, Serialize};

use crate::content;

/// Case difficulty: puzzle size plus how much lying goes on.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Level {
    /// 3x4, nobody lies.
    Easy,
    /// 4x4, one liar.
    Medium,
    /// 5x5, two liars including the culprit, subtle tells.
    Hard,
}

impl FromStr for Level {
    type Err = String;

    /// Parses `"easy"`, `"medium"`, or `"hard"` (case-insensitive).
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "easy" => Ok(Level::Easy),
            "medium" => Ok(Level::Medium),
            "hard" => Ok(Level::Hard),
            _ => Err(format!("unknown level '{s}' (easy|medium|hard)")),
        }
    }
}

impl Level {
    /// Puzzle dimensions: Easy 3x4, Medium 4x4, Hard 5x5.
    pub fn size(self) -> Size {
        match self {
            Level::Easy => Size { cats: 3, items: 4 },
            Level::Medium => Size { cats: 4, items: 4 },
            Level::Hard => Size { cats: 5, items: 5 },
        }
    }

    /// Clue difficulty handed to the generator.
    pub fn difficulty(self) -> Difficulty {
        match self {
            Level::Easy => Difficulty::Easy,
            Level::Medium => Difficulty::Medium,
            Level::Hard => Difficulty::Hard,
        }
    }

    /// Number of lying suspects.
    pub fn liars(self) -> usize {
        match self {
            Level::Easy => 0,
            Level::Medium => 1,
            Level::Hard => 2,
        }
    }

    /// Whether the culprit is always a liar and lies about the guilty category.
    pub fn culprit_lies(self) -> bool {
        self == Level::Hard
    }

    /// Whether pressing a lie produces only a subtle drift.
    pub fn subtle_tells(self) -> bool {
        self == Level::Hard
    }

    /// Lowercase name.
    pub fn name(self) -> &'static str {
        match self {
            Level::Easy => "easy",
            Level::Medium => "medium",
            Level::Hard => "hard",
        }
    }

    /// False statements per liar.
    fn lies(self) -> usize {
        match self {
            Level::Easy => 0,
            Level::Medium => 2,
            Level::Hard => 3,
        }
    }
}

/// How a suspect talks.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Mood {
    /// Helpful and polite.
    Cooperative,
    /// Jittery and apologetic.
    Nervous,
    /// Curt and resentful.
    Hostile,
    /// Rambling and eager.
    Chatty,
}

/// Everything that determines a case.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    /// Random seed.
    pub seed: u64,
    /// Difficulty.
    pub level: Level,
    /// Theme index into [`crate::theme_names`]; `None` picks from the seed.
    pub theme: Option<usize>,
    /// Adult mode: profanity, harsher crimes, innuendo.
    pub adult: bool,
}

/// One thing a suspect can say.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Statement {
    /// The underlying clue.
    pub clue: Clue,
    /// True if the clue holds under the solution.
    pub truthful: bool,
    /// True once the player has been told it.
    pub revealed: bool,
}

/// One anchor item, personified.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Suspect {
    /// Anchor item display name.
    pub name: String,
    /// `<name lowercased>@<frame.domain>`.
    pub email: String,
    /// Has false statements.
    pub liar: bool,
    /// Tone of replies.
    pub mood: Mood,
    /// Feels guilty: hedges in replies.
    pub conscience: bool,
    /// Everything they can say, shuffled.
    pub statements: Vec<Statement>,
    /// Stops replying after being wrongly accused.
    pub silenced: bool,
}

/// Theme framing resolved for this case (from content.rs; adult variants already applied).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Frame {
    /// Puzzle title, e.g. "Manor Mystery".
    pub title: String,
    /// Email domain, e.g. "thornfield.manor".
    pub domain: String,
    /// Display name of the player's boss, e.g. "DCI Marlowe".
    pub chief: String,
    /// 2-4 sentences: what happened. Never names a category.
    pub setting: String,
    /// Short noun phrase, e.g. "the theft".
    pub crime: String,
}

/// A puzzle dressed as a detective case.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Case {
    /// What generated it.
    pub settings: Settings,
    /// The underlying puzzle.
    pub puzzle: Puzzle,
    /// One per anchor item, same order.
    pub suspects: Vec<Suspect>,
    /// The guilty fact: a non-anchor entity chosen from the seed.
    pub guilty: Entity,
    /// Anchor item the solution pairs with `guilty`.
    pub culprit: usize,
    /// Theme framing.
    pub frame: Frame,
}

impl Case {
    /// Generate the case for `settings`. Deterministic.
    pub fn new(settings: Settings) -> Case {
        let level = settings.level;
        let puzzle = generate(
            level.size(),
            level.difficulty(),
            settings.seed,
            settings.theme,
        );
        let sol = &puzzle.solution;
        let (n, m) = (puzzle.n_cats(), puzzle.n_items());
        let mut rng = StdRng::seed_from_u64(settings.seed ^ 0xCA5E);

        let guilty = Entity {
            cat: rng.random_range(1..n),
            item: rng.random_range(0..m),
        };
        let culprit = sol.anchor_of(guilty);

        let mut order: Vec<usize> = (0..m).filter(|&i| i != culprit).collect();
        order.shuffle(&mut rng);
        if level.culprit_lies() {
            order.insert(0, culprit);
        } else {
            order.insert(rng.random_range(0..=order.len()), culprit);
        }
        let liars: Vec<usize> = order[..level.liars()].to_vec();

        let fd = content::frame(&puzzle.title);
        let frame = Frame {
            title: puzzle.title.clone(),
            domain: fd.domain.into(),
            chief: fd.chief.into(),
            setting: if settings.adult {
                fd.setting_adult
            } else {
                fd.setting
            }
            .into(),
            crime: if settings.adult {
                fd.crime_adult
            } else {
                fd.crime
            }
            .into(),
        };

        let mut suspects: Vec<Suspect> = puzzle.categories[0]
            .items
            .iter()
            .enumerate()
            .map(|(i, name)| Suspect {
                name: name.clone(),
                email: format!("{}@{}", name.to_lowercase(), frame.domain),
                liar: liars.contains(&i),
                mood: [
                    Mood::Cooperative,
                    Mood::Nervous,
                    Mood::Hostile,
                    Mood::Chatty,
                ][rng.random_range(0..4)],
                conscience: rng.random_bool(if i == culprit { 0.75 } else { 0.25 }),
                statements: Vec::new(),
                silenced: false,
            })
            .collect();

        // Truthful statements: every generator clue goes to the first anchor it mentions, else to
        // whoever has the fewest so far.
        for clue in &puzzle.clues {
            let owner = match clue.entities().iter().find(|e| e.cat == 0) {
                Some(e) => e.item,
                None => {
                    let min = suspects
                        .iter()
                        .map(|s| s.statements.len())
                        .min()
                        .unwrap_or(0);
                    let ties: Vec<usize> = (0..m)
                        .filter(|&i| suspects[i].statements.len() == min)
                        .collect();
                    *ties.choose(&mut rng).expect("at least one suspect")
                }
            };
            suspects[owner].statements.push(Statement {
                clue: clue.clone(),
                truthful: true,
                revealed: false,
            });
        }

        // False statements for each liar.
        let e = |cat, item| Entity { cat, item };
        let wrong = |rng: &mut StdRng, truth: usize| {
            let k = rng.random_range(1..m);
            (truth + k) % m
        };
        for &i in &liars {
            let mut lies: Vec<Clue> = Vec::new();
            if level.culprit_lies() && i == culprit {
                let w = wrong(&mut rng, guilty.item);
                lies.push(Clue::Is(e(0, i), e(guilty.cat, w)));
            }
            while lies.len() < level.lies() {
                let cat = rng.random_range(1..n);
                let lie = match rng.random_range(0..3) {
                    0 => Clue::Is(e(0, i), e(cat, wrong(&mut rng, sol.0[cat][i]))),
                    1 => Clue::IsNot(e(0, i), e(cat, sol.0[cat][i])),
                    _ => {
                        let other = (i + rng.random_range(1..m)) % m;
                        Clue::Is(e(0, other), e(cat, wrong(&mut rng, sol.0[cat][other])))
                    }
                };
                if !lie.holds(sol) && !lies.contains(&lie) {
                    lies.push(lie);
                }
            }
            suspects[i]
                .statements
                .extend(lies.into_iter().map(|clue| Statement {
                    clue,
                    truthful: false,
                    revealed: false,
                }));
        }
        for s in &mut suspects {
            s.statements.shuffle(&mut rng);
        }

        Case {
            settings,
            puzzle,
            suspects,
            guilty,
            culprit,
            frame,
        }
    }

    /// "was in the Study": guilty category phrase with the item filled in.
    pub fn guilty_fact(&self) -> String {
        self.fact(self.guilty)
    }

    /// Verb phrase for any non-anchor entity, e.g. "owns the Cat".
    pub(crate) fn fact(&self, e: Entity) -> String {
        let ph = &self.puzzle.categories[e.cat].phrase;
        let ph = if ph.is_empty() { "goes with {}" } else { ph };
        ph.replace("{}", self.puzzle.name(e))
    }

    /// Anchor category name lowercased, e.g. "guest".
    pub fn noun(&self) -> String {
        self.puzzle.categories[0].name.to_lowercase()
    }
}
