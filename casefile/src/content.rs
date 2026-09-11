// PLACEHOLDER: rewritten by the content task
//! Flavor text: theme frames, suspect voices by mood, and the chief's email templates.

use crate::Mood;

/// Framing for one theme: who the chief is, where the mail comes from, what happened.
pub struct FrameData {
    /// Theme title this frame belongs to (matches `Puzzle::title`).
    pub theme: &'static str,
    /// Email domain for the suspects.
    pub domain: &'static str,
    /// Display name of the player's boss.
    pub chief: &'static str,
    /// What happened, 2-4 sentences. Must not name a category.
    pub setting: &'static str,
    /// Adult-mode variant of `setting`.
    pub setting_adult: &'static str,
    /// Short noun phrase for the crime, e.g. "the theft".
    pub crime: &'static str,
    /// Adult-mode variant of `crime`.
    pub crime_adult: &'static str,
}

const GENERIC: FrameData = FrameData {
    theme: "",
    domain: "precinct.example",
    chief: "DCI Marlowe",
    setting: "Something went badly wrong last night and everyone who was there is now a \
              suspect. Nobody has left, nobody is talking freely, and the chief wants a name \
              before the press gets hold of it.",
    setting_adult: "Something went badly wrong last night and everyone who was there is now a \
                    suspect. Nobody has left, nobody is talking straight, and the chief wants \
                    a damn name before the press gets hold of it.",
    crime: "the incident",
    crime_adult: "the whole bloody mess",
};

const FRAMES: &[FrameData] = &[];

/// Frame for a theme title; unknown titles get a generic frame.
pub fn frame(theme: &str) -> &'static FrameData {
    FRAMES.iter().find(|f| f.theme == theme).unwrap_or(&GENERIC)
}

/// How a suspect of a given mood talks.
pub struct VoiceData {
    /// The mood this voice belongs to.
    pub mood: Mood,
    /// Reply openers.
    pub openers: &'static [&'static str],
    /// Adult-mode openers.
    pub openers_adult: &'static [&'static str],
    /// Reply closers (sign-offs).
    pub closers: &'static [&'static str],
    /// Adult-mode closers.
    pub closers_adult: &'static [&'static str],
    /// Non-answers when the suspect has nothing left on a topic.
    pub dodges: &'static [&'static str],
    /// Adult-mode dodges.
    pub dodges_adult: &'static [&'static str],
    /// Pressed on a truthful statement; "{stmt}" is replaced with the verbatim statement.
    pub press_true: &'static [&'static str],
    /// Pressed on a false statement; "{stmt}" is replaced with a re-rendered variant.
    pub press_false: &'static [&'static str],
    /// Subtle version of `press_false` for Hard.
    pub press_false_subtle: &'static [&'static str],
    /// Extra hedge sentence when `conscience` is set.
    pub conscience: &'static [&'static str],
    /// Adult-mode hedge.
    pub conscience_adult: &'static [&'static str],
}

const VOICES: [VoiceData; 4] = [
    VoiceData {
        mood: Mood::Cooperative,
        openers: &[
            "Of course, happy to help.",
            "Thanks for asking me directly.",
        ],
        openers_adult: &[
            "Of course, happy to help.",
            "Thanks for asking me straight.",
        ],
        closers: &["Let me know if there's anything else.", "Hope that helps."],
        closers_adult: &["Let me know if there's anything else.", "Hope that helps."],
        dodges: &[
            "I've told you everything I know about that.",
            "I'm afraid I can't add anything there.",
        ],
        dodges_adult: &[
            "I've told you everything I know about that.",
            "I've got nothing else on that, honestly.",
        ],
        press_true: &["As I said: {stmt}", "I'll stand by it. {stmt}"],
        press_false: &[
            "Right, yes. {stmt} That's what I meant.",
            "Let me rephrase. {stmt}",
        ],
        press_false_subtle: &["{stmt}", "Just as I said. {stmt}"],
        conscience: &[
            "I do feel awful about all this.",
            "I keep going over that night.",
        ],
        conscience_adult: &[
            "I feel bloody awful about all this.",
            "I keep going over that night.",
        ],
    },
    VoiceData {
        mood: Mood::Nervous,
        openers: &["Oh. Um, okay.", "Sorry, I didn't expect to hear from you."],
        openers_adult: &["Oh. Um, okay.", "Sorry, I didn't expect to hear from you."],
        closers: &[
            "Is that all? I hope that's all.",
            "Please don't read anything into this.",
        ],
        closers_adult: &[
            "Is that all? I hope that's all.",
            "Please don't read anything into this.",
        ],
        dodges: &[
            "I, I don't know anything else about that.",
            "Can we not talk about that?",
        ],
        dodges_adult: &[
            "I, I don't know anything else about that.",
            "Can we not talk about that?",
        ],
        press_true: &["I already said. {stmt}", "Why are you asking again? {stmt}"],
        press_false: &["Wait, I mean, {stmt} Yes.", "Sorry, I got muddled. {stmt}"],
        press_false_subtle: &["{stmt}", "Like I said. {stmt}"],
        conscience: &[
            "I should have done something.",
            "I can't sleep since it happened.",
        ],
        conscience_adult: &[
            "I should have done something.",
            "I can't sleep since it happened.",
        ],
    },
    VoiceData {
        mood: Mood::Hostile,
        openers: &["Fine.", "You again."],
        openers_adult: &["Fine.", "What the hell do you want now?"],
        closers: &["We're done here.", "Don't write again."],
        closers_adult: &["We're done here.", "Piss off."],
        dodges: &["No comment.", "Ask someone who cares."],
        dodges_adult: &["No comment.", "Ask someone who gives a damn."],
        press_true: &["I said what I said. {stmt}", "Read it again. {stmt}"],
        press_false: &["If you must. {stmt} Satisfied?", "Fine. {stmt} Happy now?"],
        press_false_subtle: &["{stmt}", "Same answer. {stmt}"],
        conscience: &[
            "Not that it matters now.",
            "It's not like I wanted any of this.",
        ],
        conscience_adult: &[
            "Not that it matters now.",
            "It's not like I wanted any of this.",
        ],
    },
    VoiceData {
        mood: Mood::Chatty,
        openers: &["Oh, finally someone asks!", "I was hoping you'd write."],
        openers_adult: &["Oh, finally someone asks!", "I was hoping you'd write."],
        closers: &[
            "Anyway, write back soon!",
            "There's more where that came from.",
        ],
        closers_adult: &[
            "Anyway, write back soon!",
            "There's more where that came from.",
        ],
        dodges: &[
            "Oh, I've run out of things to say on that, and that never happens.",
            "You know, on that one I've honestly got nothing. Ask me something else!",
        ],
        dodges_adult: &[
            "Oh, I've run out of things to say on that, and that never happens.",
            "You know, on that one I've honestly got nothing. Ask me something else!",
        ],
        press_true: &["Word for word, then: {stmt}", "Happily. {stmt}"],
        press_false: &[
            "Well, more or less. {stmt} Something like that.",
            "Oh, {stmt} Did I say otherwise?",
        ],
        press_false_subtle: &["{stmt}", "Sure. {stmt}"],
        conscience: &[
            "I just wish I'd said something sooner.",
            "It weighs on me, honestly.",
        ],
        conscience_adult: &[
            "I just wish I'd said something sooner.",
            "It weighs on me, honestly.",
        ],
    },
];

/// Voice for a mood.
pub fn voice(mood: Mood) -> &'static VoiceData {
    VOICES
        .iter()
        .find(|v| v.mood == mood)
        .expect("every mood has a voice")
}

/// Chief email templates. Placeholders: `{setting}` `{crime}` `{names}` `{fact}` `{noun}`
/// `{rank}` `{epilogue}` `{culprit}` `{chief}`.
pub struct ChiefData {
    /// Opening briefing.
    pub intro: &'static str,
    /// Sent once when more than half the truthful statements are in.
    pub nudge: &'static str,
    /// Sent after a wrong accusation.
    pub wrong: &'static str,
    /// Sent after the right accusation.
    pub solved: &'static str,
}

const CLEAN: ChiefData = ChiefData {
    intro: "{setting}\n\nWe know one thing for certain about {crime}: the {noun} responsible \
            {fact}. Work out who that is. Present: {names}. Email them, compare their stories, \
            and press anyone who sounds shaky. When you're sure, send me an accusation.\n\n{chief}",
    nudge: "You're past the halfway mark. Keep the pressure on.\n\n{chief}",
    wrong: "That was the wrong {noun}, and now they won't talk to us. It goes on your \
            record. Try again.\n\n{chief}",
    solved: "That's the one. {culprit} {fact}, and that settles {crime}.\n\n{epilogue}\n\n\
             Final rank: {rank}.\n\n{chief}",
};

const ADULT: ChiefData = ChiefData {
    intro: "{setting}\n\nOne thing is certain about {crime}: the {noun} responsible {fact}. \
            Find out who the hell that is. Present: {names}. Email them, compare their \
            stories, and lean on anyone who sounds shaky. When you're sure, send me an \
            accusation.\n\n{chief}",
    nudge: "You're past halfway. Don't get comfortable.\n\n{chief}",
    wrong: "Wrong {noun}, and now they've lawyered up. That goes on your record. Again.\n\n{chief}",
    solved: "Got the bastard. {culprit} {fact}, and that settles {crime}.\n\n{epilogue}\n\n\
             Final rank: {rank}.\n\n{chief}",
};

/// Chief templates for the mode.
pub fn chief(adult: bool) -> &'static ChiefData {
    if adult { &ADULT } else { &CLEAN }
}
