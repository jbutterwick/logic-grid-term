//! The inbox state machine: questions, delayed replies, the chief, and accusations (D1.5-D1.7).

use logicgrid::{Cat, Entity, Grid};
use serde::{Deserialize, Serialize};

use crate::content::{chief, voice};
use crate::text::render;
use crate::{Case, Settings};

/// Something the player can ask a suspect.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Question {
    /// The suspect's own facts.
    Own,
    /// Another suspect, by index.
    About(usize),
    /// A non-anchor category.
    Category(Cat),
    /// Repeat the last answer.
    Press,
}

impl Question {
    /// Menu label: "Where were you that night?", "What do you know about Crane?",
    /// "Tell me about the Motive.", "Press on your last answer."
    pub fn label(&self, case: &Case) -> String {
        match *self {
            Question::Own => "Where were you that night?".into(),
            Question::About(s) => format!("What do you know about {}?", case.suspects[s].name),
            Question::Category(c) => {
                format!("Tell me about the {}.", case.puzzle.categories[c].name)
            }
            Question::Press => "Press on your last answer.".into(),
        }
    }
}

/// One message in a thread.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Email {
    /// Sender display name or address.
    pub from: String,
    /// Subject line.
    pub subject: String,
    /// Body text.
    pub body: String,
    /// Tick it landed in the inbox.
    pub tick: u32,
    /// Read by the player.
    pub read: bool,
    /// Sent by the player.
    pub outgoing: bool,
}

/// Final rank by wrong accusations: 0, 1, 2+.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Rank {
    /// No wrong accusations.
    Inspector,
    /// One.
    Detective,
    /// Two or more.
    Constable,
}

/// A case in play.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Game {
    /// The case.
    pub case: Case,
    /// Action counter; mail is delivered by tick.
    pub tick: u32,
    /// Thread with the chief; starts with the intro email.
    pub chief: Vec<Email>,
    /// One thread per suspect.
    pub threads: Vec<Vec<Email>>,
    /// Notepad grid marks.
    pub grid: Grid,
    /// Free-text notepad.
    pub notes: String,
    /// Wrong accusations so far.
    pub wrong: u32,
    /// Presses so far.
    pub presses: u32,
    /// Case closed.
    pub solved: bool,
    /// Replies not yet delivered: (due tick, thread, email). `None` = chief.
    pending: Vec<(u32, Option<usize>, Email)>,
    /// Index of the last revealed statement per suspect.
    last: Vec<Option<usize>>,
    /// The chief's halfway nudge has been sent.
    nudged: bool,
}

impl Game {
    /// Start a game: generate the case and receive the chief's briefing.
    pub fn new(settings: Settings) -> Game {
        let case = Case::new(settings);
        let m = case.suspects.len();
        let mut g = Game {
            grid: Grid::new(&case.puzzle),
            case,
            tick: 0,
            chief: Vec::new(),
            threads: vec![Vec::new(); m],
            notes: String::new(),
            wrong: 0,
            presses: 0,
            solved: false,
            pending: Vec::new(),
            last: vec![None; m],
            nudged: false,
        };
        let intro = g.fill(chief(g.case.settings.adult).intro);
        g.chief.push(g.email(None, intro, false));
        g
    }

    /// Own, About(every other suspect), Category(every non-anchor category), then Press if
    /// the suspect has answered at least once. Empty if the suspect is silenced.
    pub fn questions(&self, suspect: usize) -> Vec<Question> {
        if self.case.suspects[suspect].silenced {
            return Vec::new();
        }
        let mut qs = vec![Question::Own];
        qs.extend(
            (0..self.case.suspects.len())
                .filter(|&s| s != suspect)
                .map(Question::About),
        );
        qs.extend((1..self.case.puzzle.n_cats()).map(Question::Category));
        if self.last[suspect].is_some() {
            qs.push(Question::Press);
        }
        qs
    }

    /// Append the outgoing email, queue the reply for tick+1, advance the tick, deliver due mail.
    pub fn ask(&mut self, suspect: usize, q: Question) {
        let out = self.email(Some(suspect), q.label(&self.case), true);
        self.threads[suspect].push(out);
        let reply = self.reply(suspect, q);
        self.tick += 1;
        let due = self.tick + 1;
        let mail = self.email(Some(suspect), reply, false);
        self.pending.push((due, Some(suspect), mail));
        let (done, total) = self.progress();
        if !self.nudged && done * 2 > total {
            self.nudged = true;
            let body = self.fill(chief(self.case.settings.adult).nudge);
            let mail = self.email(None, body, false);
            self.pending.push((due, None, mail));
        }
        self.deliver();
    }

    /// Advance the tick and deliver due mail.
    pub fn wait(&mut self) {
        self.tick += 1;
        self.deliver();
    }

    /// True when suspect == culprit and fact == guilty. Wrong: wrong += 1, accused silenced,
    /// chief sends `wrong`. Right: solved = true, chief sends `solved` with rank and epilogue.
    pub fn accuse(&mut self, suspect: usize, fact: Entity) -> bool {
        let name = self.case.suspects[suspect].name.clone();
        let body = format!(
            "I accuse {name}: the {} who {}.",
            self.case.noun(),
            self.case.fact(fact)
        );
        let out = self.email(None, body, true);
        self.chief.push(out);
        self.tick += 1;
        self.deliver();
        let right = suspect == self.case.culprit && fact == self.case.guilty;
        let templates = chief(self.case.settings.adult);
        let body = if right {
            self.solved = true;
            self.fill(templates.solved)
        } else {
            self.wrong += 1;
            self.case.suspects[suspect].silenced = true;
            self.fill(templates.wrong)
        };
        let mail = self.email(None, body, false);
        self.chief.push(mail);
        right
    }

    /// (truthful statements revealed, truthful statements total).
    pub fn progress(&self) -> (usize, usize) {
        let truthful = self
            .case
            .suspects
            .iter()
            .flat_map(|s| &s.statements)
            .filter(|st| st.truthful);
        truthful
            .clone()
            .fold((0, 0), |(r, t), st| (r + usize::from(st.revealed), t + 1))
    }

    /// Rank from wrong accusations.
    pub fn rank(&self) -> Rank {
        match self.wrong {
            0 => Rank::Inspector,
            1 => Rank::Detective,
            _ => Rank::Constable,
        }
    }

    /// Unread count; None = chief thread.
    pub fn unread(&self, thread: Option<usize>) -> usize {
        self.thread(thread).iter().filter(|e| !e.read).count()
    }

    /// Mark every email in the thread read.
    pub fn mark_read(&mut self, thread: Option<usize>) {
        let t = match thread {
            None => &mut self.chief,
            Some(i) => &mut self.threads[i],
        };
        t.iter_mut().for_each(|e| e.read = true);
    }

    /// Full solution as prose, one sentence per suspect, e.g.
    /// "Crane was in the Study, was last seen at Midnight, and is driven by Greed."
    pub fn epilogue(&self) -> String {
        let p = &self.case.puzzle;
        self.case
            .suspects
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let facts: Vec<String> = (1..p.n_cats())
                    .map(|c| {
                        self.case.fact(Entity {
                            cat: c,
                            item: p.solution.0[c][i],
                        })
                    })
                    .collect();
                let joined = match facts.len() {
                    1 => facts[0].clone(),
                    2 => format!("{} and {}", facts[0], facts[1]),
                    k => format!("{}, and {}", facts[..k - 1].join(", "), facts[k - 1]),
                };
                format!("{} {joined}.", s.name)
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Serialize the whole game.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("game serializes")
    }

    /// Restore a game from [`Game::to_json`].
    pub fn from_json(s: &str) -> Result<Game, serde_json::Error> {
        serde_json::from_str(s)
    }

    fn thread(&self, thread: Option<usize>) -> &[Email] {
        match thread {
            None => &self.chief,
            Some(i) => &self.threads[i],
        }
    }

    /// Build an email in `thread` at the current tick.
    fn email(&self, thread: Option<usize>, body: String, outgoing: bool) -> Email {
        let from = match (outgoing, thread) {
            (true, _) => "you".to_string(),
            (false, None) => self.case.frame.chief.clone(),
            (false, Some(i)) => self.case.suspects[i].email.clone(),
        };
        Email {
            from,
            subject: format!("Re: {}", self.case.frame.crime),
            body,
            tick: self.tick,
            read: outgoing,
            outgoing,
        }
    }

    /// Move every pending email that is due into its thread, in due order.
    fn deliver(&mut self) {
        let tick = self.tick;
        let (due, later): (Vec<_>, Vec<_>) = std::mem::take(&mut self.pending)
            .into_iter()
            .partition(|p| p.0 <= tick);
        self.pending = later;
        for (at, thread, mut mail) in due {
            mail.tick = at;
            match thread {
                None => self.chief.push(mail),
                Some(i) => self.threads[i].push(mail),
            }
        }
    }

    /// Rendering variant for suspect `s`'s statement `idx`; stable so presses can repeat it.
    fn variant(s: usize, idx: usize) -> usize {
        s * 3 + idx
    }

    /// Compose the suspect's reply body for `q`, revealing a statement if one matches.
    fn reply(&mut self, suspect: usize, q: Question) -> String {
        let adult = self.case.settings.adult;
        let subtle = self.case.settings.level.subtle_tells();
        let p = &self.case.puzzle;
        let s = &self.case.suspects[suspect];
        let v = voice(s.mood);
        let salt = self.tick as usize * 31 + suspect * 7;
        let pick = |clean: &'static [&'static str], hot: &'static [&'static str], k: usize| {
            let list = if adult && !hot.is_empty() { hot } else { clean };
            list[k % list.len()]
        };
        let hit = |st: &crate::Statement| match q {
            Question::Own => st
                .clue
                .entities()
                .iter()
                .any(|e| e.cat == 0 && e.item == suspect),
            Question::About(o) => st.clue.entities().iter().any(|e| e.cat == 0 && e.item == o),
            Question::Category(c) => st.clue.entities().iter().any(|e| e.cat == c),
            Question::Press => false,
        };
        let middle = match q {
            Question::Press => match self.last[suspect] {
                Some(i) => {
                    self.presses += 1;
                    let st = &s.statements[i];
                    let var = Self::variant(suspect, i);
                    let (list, stmt) = if st.truthful {
                        (v.press_true, render(&st.clue, p, Some(suspect), var))
                    } else {
                        let list = if subtle {
                            v.press_false_subtle
                        } else {
                            v.press_false
                        };
                        (list, render(&st.clue, p, Some(suspect), var + 1))
                    };
                    list[salt % list.len()].replace("{stmt}", &stmt)
                }
                None => pick(v.dodges, v.dodges_adult, salt).to_string(),
            },
            _ => match s.statements.iter().position(|st| !st.revealed && hit(st)) {
                Some(i) => {
                    let text = render(
                        &s.statements[i].clue,
                        p,
                        Some(suspect),
                        Self::variant(suspect, i),
                    );
                    self.case.suspects[suspect].statements[i].revealed = true;
                    self.last[suspect] = Some(i);
                    text
                }
                None => pick(v.dodges, v.dodges_adult, salt).to_string(),
            },
        };
        let s = &self.case.suspects[suspect];
        let hedge = if s.conscience {
            format!(" {}", pick(v.conscience, v.conscience_adult, salt + 1))
        } else {
            String::new()
        };
        format!(
            "{} {middle}{hedge}\n\n{}",
            pick(v.openers, v.openers_adult, salt),
            pick(v.closers, v.closers_adult, salt + 2)
        )
    }

    /// Fill a chief template's placeholders.
    fn fill(&self, t: &str) -> String {
        let c = &self.case;
        let names: Vec<&str> = c.suspects.iter().map(|s| s.name.as_str()).collect();
        let names = match names.len() {
            0 => String::new(),
            1 => names[0].to_string(),
            k => format!("{} and {}", names[..k - 1].join(", "), names[k - 1]),
        };
        t.replace("{setting}", &c.frame.setting)
            .replace("{crime}", &c.frame.crime)
            .replace("{names}", &names)
            .replace("{fact}", &c.guilty_fact())
            .replace("{noun}", &c.noun())
            .replace("{rank}", &format!("{:?}", self.rank()))
            .replace("{epilogue}", &self.epilogue())
            .replace("{culprit}", &c.suspects[c.culprit].name)
            .replace("{chief}", &c.frame.chief)
    }
}
