//! Flavor text: theme frames, suspect voices by mood, and the chief's email templates (D1.8).
//!
//! Every string is a plain `&'static str`. The only runtime substitution is the placeholder
//! set documented on [`ChiefData`] and the `{stmt}` marker in the press lists. Settings never
//! name a category or an item, because the generator trims both per case.

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

const FRAMES: &[FrameData] = &[
    FrameData {
        theme: "Apartment Block",
        domain: "maplestreet.walkup",
        chief: "Superintendent Okafor",
        setting: "The tenants of the Maple Street walk-up pooled three months of rent for the \
                  roof repair and kept it in a biscuit tin behind the boiler. This morning the \
                  tin was on the landing, empty, and the landlord is threatening to evict the \
                  lot of them. Everyone swears they were home all evening. The building \
                  association has asked you to sort it out before the eviction notices go up.",
        setting_adult: "The tenants of the Maple Street walk-up pooled three months of rent for \
                        the roof repair and kept it in a biscuit tin behind the boiler. This \
                        morning the tin was on the landing, empty, and the landlord is \
                        threatening to throw the whole bloody lot of them out. Everyone swears \
                        they were home all evening, which is a hell of a coincidence. The \
                        building association wants a name before the landlord changes the locks.",
        crime: "the missing rent money",
        crime_adult: "the rent-money job",
    },
    FrameData {
        theme: "Dinner Party",
        domain: "hostess.dining",
        chief: "Inspector Vasquez",
        setting: "Somewhere between the main course and dessert, the hostess's engagement ring \
                  vanished from the saucer by the sink where she always leaves it while she \
                  cooks. The front door was locked all night. Nobody has gone home, the coffee \
                  has gone cold, and the hostess is trying very hard to keep smiling. She has \
                  asked you to find the ring quietly.",
        setting_adult: "Somewhere between the main course and dessert, the hostess's engagement \
                        ring vanished from the saucer by the sink where she always leaves it \
                        while she cooks. The front door was locked all night, so whoever took it \
                        is still sitting at the damn table. Nobody has gone home, the coffee has \
                        gone cold, and the hostess has stopped pretending to smile. She wants \
                        the ring back and she wants to know who to never invite again.",
        crime: "the missing ring",
        crime_adult: "the ring that walked off",
    },
    FrameData {
        theme: "Marathon",
        domain: "raceday.org",
        chief: "Race Director Lindqvist",
        setting: "The winner's trophy was lifted from the awards tent while the medals were \
                  being handed out, and the timing chips for the top five were wiped from the \
                  results server minutes later. The organizers are refusing to certify any \
                  result until the trophy turns up. Every contender was in the athletes' \
                  village at the time, and every one of them has a story. The race committee \
                  has asked you to find out whose story is fiction.",
        setting_adult: "The winner's trophy was lifted from the awards tent while the medals \
                        were being handed out, and the timing chips for the top five were wiped \
                        from the results server minutes later. The organizers are refusing to \
                        certify a damn thing until the trophy turns up. Every contender was in \
                        the athletes' village at the time, and every one of them has a story. \
                        The race committee wants to know who's lying and who's just sore.",
        crime: "the theft of the trophy",
        crime_adult: "the trophy grab",
    },
    FrameData {
        theme: "Flea Market",
        domain: "sundaymarket.co",
        chief: "Market Warden Adeyemi",
        setting: "A dealer's strongbox went missing from under a folding trestle at the Sunday \
                  market, and with it the morning's takings and a signed receipt book. The \
                  dealer had only turned away to haggle. Five people had been circling that \
                  pitch all morning, and each of them left with something under their arm. The \
                  market warden wants the box back before the dealer calls the police.",
        setting_adult: "A dealer's strongbox went missing from under a folding trestle at the \
                        Sunday market, and with it the morning's takings and a receipt book the \
                        dealer would rather nobody read. The dealer had only turned away to \
                        haggle. Five people had been circling that pitch all morning, and every \
                        one of them left with something under their arm. The market warden \
                        wants the box back before the dealer does something stupid about it.",
        crime: "the missing strongbox",
        crime_adult: "the strongbox lift",
    },
    FrameData {
        theme: "Cul-de-Sac",
        domain: "willowclose.net",
        chief: "Sergeant Brannigan",
        setting: "Someone put sugar in the fuel tank of the community minibus the night before \
                  the seniors' outing, and the residents' association noticeboard was torn \
                  down for good measure. Willow Close only has five homes, and every one of \
                  them had a light on past midnight. The association chair is furious and \
                  wants it settled before the next meeting.",
        setting_adult: "Someone put sugar in the fuel tank of the community minibus the night \
                        before the seniors' outing, then keyed the chairman's paintwork and \
                        tore down the residents' noticeboard for good measure. Willow Close only \
                        has five homes, and every one of them had a light on past midnight. The \
                        association chair is livid and wants a name before the next meeting \
                        turns into a brawl.",
        crime: "the sabotaged minibus",
        crime_adult: "the minibus sabotage",
    },
    FrameData {
        theme: "Harbor",
        domain: "saltwharf.harbor",
        chief: "Harbormaster Quist",
        setting: "The harbor's brass ship's bell, cast in 1871 and rung at every homecoming \
                  since, was unbolted from the wharf head during the night tide. Five crews \
                  were moored within sight of it, and nobody heard a thing over the wind. The \
                  harbormaster will not sign anyone out until the bell is back. You have until \
                  the weather turns.",
        setting_adult: "The harbor's brass ship's bell, cast in 1871 and rung at every \
                        homecoming since, was unbolted from the wharf head during the night \
                        tide. Five crews were moored within sight of it, and every damn one of \
                        them claims to have slept through the wind. The harbormaster won't sign \
                        a soul out until the bell is back, and the crews are getting ugly about \
                        it. You have until the weather turns.",
        crime: "the theft of the harbor bell",
        crime_adult: "the bell heist",
    },
    FrameData {
        theme: "High School",
        domain: "ridgeview.edu",
        chief: "Vice Principal Haddad",
        setting: "The answer key for the final exams was photographed in the staff office and \
                  posted to an anonymous account before first period. The office was open for \
                  exactly twenty minutes, and five pupils were signed in during that window. \
                  The vice principal wants a name before the exam board hears about it.",
        setting_adult: "The answer key for the final exams was photographed in the staff office \
                        and posted to an anonymous account before first period, along with a \
                        very unflattering photo of the principal. The office was open for \
                        exactly twenty minutes, and five pupils were signed in during that \
                        window. The vice principal wants a name before the exam board hears \
                        about it, and honestly wants the photo taken down more.",
        crime: "the leaked exam",
        crime_adult: "the exam leak",
    },
    FrameData {
        theme: "Heist Crew",
        domain: "nightwork.onion",
        chief: "the Fixer",
        setting: "The job went clean, but the payoff was light by a third when it was divided \
                  in the lockup. Someone skimmed on the way out. The Fixer has closed the \
                  lockup with everyone still inside and has hired you, an outside pair of eyes, \
                  to find the skimmer before the crew starts settling it among themselves.",
        setting_adult: "The job went clean, but the payoff was light by a third when it was \
                        divided in the lockup, and the driver's body was in the trunk instead \
                        of behind the wheel. Someone skimmed on the way out and made damn sure \
                        the driver couldn't say who. The Fixer has closed the lockup with \
                        everyone still inside and has hired you, an outside pair of eyes, to \
                        find the rat before the crew finds their own answer with a tire iron.",
        crime: "the skimmed payoff",
        crime_adult: "the rat in the crew",
    },
    FrameData {
        theme: "Haunted Hotel",
        domain: "grandhollow.hotel",
        chief: "Night Manager Vane",
        setting: "The Grand Hollow's last living guest checked out at dawn, white as a sheet, \
                  after something dragged every painting in the east wing off its hook in a single \
                  night. The resident spirits have haunted this hotel for a century without so \
                  much as cracking a teacup. The night manager wants to know which of them has \
                  broken the truce, and has asked you, a medium of some reputation, to hold the \
                  interviews.",
        setting_adult: "The Grand Hollow's last living guest checked out at dawn, white as a \
                        sheet, after something dragged every painting in the east wing off its \
                        hook and scrawled his name in the frost on the windows. The resident \
                        spirits have haunted this hotel for a century without so much as \
                        cracking a teacup. The night manager wants to know which of them has \
                        broken the damn truce, and has asked you, a medium of some reputation, \
                        to hold the interviews before the place ends up on the internet.",
        crime: "the breaking of the truce",
        crime_adult: "the broken truce",
    },
    FrameData {
        theme: "Space Station",
        domain: "halcyon.station",
        chief: "Commander Ishikawa",
        setting: "The oxygen scrubber's backup cartridge was pulled from its rack and vented \
                  into space through the waste airlock during the sleep cycle. The station has \
                  ninety hours of margin and the supply shuttle is a hundred hours out. Five \
                  people were aboard, and the airlock log has been wiped. The commander wants \
                  to know who, before deciding whether to trust the rest of them.",
        setting_adult: "The oxygen scrubber's backup cartridge was pulled from its rack and \
                        vented into space through the waste airlock during the sleep cycle, and \
                        someone scratched a countdown into the galley wall. The station has \
                        ninety hours of margin and the supply shuttle is a hundred hours out. \
                        Five people were aboard and the airlock log has been wiped. The \
                        commander wants a name before the station runs out of both air and \
                        patience.",
        crime: "the vented cartridge",
        crime_adult: "the airlock sabotage",
    },
    FrameData {
        theme: "Bake-Off",
        domain: "goldenwhisk.tv",
        chief: "Head Judge Oyelaran",
        setting: "Five contestants left their showstoppers proving overnight in the marquee. By \
                  morning one had collapsed, and the judges found a scattering of salt where \
                  the sugar should have been. The tent was locked and only the contestants had \
                  keys. Filming is on hold until the producers know who tampered with a rival's \
                  work.",
        setting_adult: "Five contestants left their showstoppers proving overnight in the \
                        marquee. By morning one had collapsed, another had been mysteriously \
                        bitten, and the judges found salt where the sugar should have been. The \
                        tent was locked and only the contestants had keys. Filming is on hold \
                        until the producers know who screwed with a rival's work, and the \
                        contestants are already lawyering up over the edit.",
        crime: "the sabotaged showstopper",
        crime_adult: "the showstopper sabotage",
    },
    FrameData {
        theme: "Band Tour",
        domain: "vanlife.tour",
        chief: "Tour Manager Delacroix",
        setting: "The band's vintage amplifier, the one that has played every show for a \
                  decade, was found in the loading dock with its valves smashed the morning of \
                  the last date of the tour. The van was locked and only the band could get in. \
                  Tonight's show is sold out and the tour manager wants to know who did it \
                  before the encore.",
        setting_adult: "The band's vintage amplifier, the one that has played every show for a \
                        decade, was found in the loading dock with its valves smashed and a note \
                        reading 'quit while you're behind' taped to the grille. The van was \
                        locked and only the band could get in. Tonight's show is sold out, the \
                        singer has been drunk since breakfast, and the tour manager wants to \
                        know which of these idiots did it before the encore.",
        crime: "the smashed amplifier",
        crime_adult: "the amp job",
    },
    FrameData {
        theme: "Garden Club",
        domain: "allotments.club",
        chief: "Club Secretary Pembridge",
        setting: "The allotment society's show entries were slashed and salted in the night, \
                  the week before the county judging. Five members hold keys to the gate. The \
                  club secretary wants it settled quietly before the county committee arrives.",
        setting_adult: "The allotment society's show entries were slashed and salted in the \
                        night, the week before the county judging, and someone left a very \
                        rude drawing on the committee shed. Five members hold keys to the gate \
                        and every one of them has held a grudge for years. The club secretary \
                        wants it settled before the county committee arrives and before \
                        someone gets a trowel in the back.",
        crime: "the ruined show entries",
        crime_adult: "the allotment vandalism",
    },
    FrameData {
        theme: "Manor Mystery",
        domain: "thornfield.manor",
        chief: "DCI Marlowe",
        setting: "Lord Thornfield's will was taken from the locked bureau during the storm, \
                  and the only copy went with it. Five house guests were stranded overnight by \
                  the flooded lane, and the servants were all accounted for. His lordship is \
                  not speaking to anyone. The family has asked for a discreet detective.",
        setting_adult: "Lord Thornfield was found at the foot of the great staircase during the \
                        storm with his neck broken and his new will gone from the locked \
                        bureau. Five house guests were stranded overnight by the flooded lane, \
                        and the servants were all accounted for. Every one of them stood to gain, \
                        every one of them is lying about something, and the family wants it done \
                        quietly before the papers get wind of it.",
        crime: "the theft of the will",
        crime_adult: "the death of Lord Thornfield",
    },
    FrameData {
        theme: "Coffee Shop",
        domain: "grindhouse.cafe",
        chief: "Owner Castellano",
        setting: "The Grind House tip jar, a full month's worth saved for the barista's \
                  surgery, was emptied during the morning rush. The counter camera was \
                  unplugged. Five morning fixtures were in at the time, and the owner has \
                  quietly asked you to find out which one before the staff find out for \
                  themselves.",
        setting_adult: "The Grind House tip jar, a full month's worth saved for the barista's \
                        surgery, was emptied during the morning rush, and someone left a fake \
                        twenty in its place as a joke. The counter camera was unplugged. Five \
                        morning fixtures were in at the time, and the owner has asked you to \
                        find out which one before the barista finds out and someone gets a \
                        scalding.",
        crime: "the emptied tip jar",
        crime_adult: "the tip-jar job",
    },
];

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
            "I've been hoping someone would ask.",
            "Certainly. I'll tell you what I can.",
            "Good to hear from you. Here's what I know.",
            "I want this cleared up as much as anyone.",
        ],
        openers_adult: &[
            "Of course, happy to help.",
            "Thanks for asking me straight.",
            "I've been hoping someone would ask.",
            "Certainly. I'll tell you what I can.",
            "Good to hear from you. Here's what I know.",
            "I want this cleared up as much as anyone, believe me.",
        ],
        closers: &[
            "Let me know if there's anything else.",
            "Hope that helps.",
            "Do write again if I can be useful.",
            "I'm around all day if you need more.",
            "Good luck. I mean that.",
        ],
        closers_adult: &[
            "Let me know if there's anything else.",
            "Hope that helps.",
            "Write again if I can be useful.",
            "I'm around all day if you need more.",
            "Good luck. I mean that, no matter how this ends.",
        ],
        dodges: &[
            "I've told you everything I know about that.",
            "I'm afraid I can't add anything there.",
            "I wish I could help on that one, but I genuinely don't know.",
            "That's outside what I saw, I'm sorry.",
            "I've nothing more on that. Ask me about something else?",
        ],
        dodges_adult: &[
            "I've told you everything I know about that.",
            "I've got nothing else on that, honestly.",
            "I wish I could help on that one, but I genuinely don't know.",
            "That's outside what I saw, sorry.",
            "I've nothing more on that. Ask me something else?",
        ],
        press_true: &[
            "As I said: {stmt}",
            "I'll stand by it. {stmt}",
            "Happy to repeat it. {stmt}",
            "Yes, that's right. {stmt}",
        ],
        press_false: &[
            "Right, yes. {stmt} That's what I meant.",
            "Let me rephrase. {stmt}",
            "Sorry, I should have been clearer. {stmt} Yes.",
            "Oh, did I say that? {stmt} That's the one.",
        ],
        press_false_subtle: &[
            "{stmt}",
            "Just as I said. {stmt}",
            "As far as I recall, {stmt}",
            "Certainly. {stmt}",
        ],
        conscience: &[
            "I do feel awful about all this.",
            "I keep going over that night.",
            "I only wish I'd paid more attention.",
            "None of us are sleeping well, I think.",
        ],
        conscience_adult: &[
            "I feel bloody awful about all this.",
            "I keep going over that night.",
            "I only wish I'd paid more damn attention.",
            "None of us are sleeping well, I think.",
        ],
    },
    VoiceData {
        mood: Mood::Nervous,
        openers: &[
            "Oh. Um, okay.",
            "Sorry, I didn't expect to hear from you.",
            "Right. Yes. Sorry, give me a second.",
            "Is this about the other night? It is, isn't it.",
            "I'll answer, I just, sorry. Okay.",
            "You want me. Alright. Alright.",
        ],
        openers_adult: &[
            "Oh. Um, okay.",
            "Sorry, I didn't expect to hear from you.",
            "Right. Yes. Sorry, give me a second.",
            "Oh hell, this is about the other night, isn't it.",
            "I'll answer, I just, sorry. Shit. Okay.",
            "You want me. Alright. Damn it. Alright.",
        ],
        closers: &[
            "Is that all? I hope that's all.",
            "Please don't read anything into this.",
            "I'm not in trouble, am I?",
            "Sorry. Sorry, I'm just tired.",
            "Please don't tell the others I wrote back.",
        ],
        closers_adult: &[
            "Is that all? I hope that's all.",
            "Please don't read anything into this.",
            "I'm not in trouble, am I? Shit, I'm in trouble.",
            "Sorry. Sorry, I'm just bloody exhausted.",
            "Please don't tell the others I wrote back.",
        ],
        dodges: &[
            "I, I don't know anything else about that.",
            "Can we not talk about that?",
            "I don't remember. I honestly don't, I've tried.",
            "Why are you asking me about that? I don't know.",
            "That's, no. I've got nothing on that. Nothing.",
        ],
        dodges_adult: &[
            "I, I don't know anything else about that.",
            "Can we not talk about that? Please?",
            "I don't remember. I honestly don't, I've bloody tried.",
            "Why the hell are you asking me about that? I don't know.",
            "That's, no. I've got nothing on that. Nothing, I swear.",
        ],
        press_true: &[
            "I already said. {stmt}",
            "Why are you asking again? {stmt}",
            "It's the truth, I promise. {stmt}",
            "I'll say it as many times as you want. {stmt}",
        ],
        press_false: &[
            "Wait, I mean, {stmt} Yes.",
            "Sorry, I got muddled. {stmt}",
            "No, no, that came out wrong. {stmt} That's it.",
            "Did I say that? I meant, um. {stmt}",
        ],
        press_false_subtle: &[
            "{stmt}",
            "Like I said. {stmt}",
            "I believe {stmt}",
            "Yes. {stmt}",
        ],
        conscience: &[
            "I should have done something.",
            "I can't sleep since it happened.",
            "I keep thinking it's partly my fault.",
            "I don't feel good about any of this.",
        ],
        conscience_adult: &[
            "I should have done something, damn it.",
            "I can't sleep since it happened.",
            "I keep thinking it's partly my fault, and it's killing me.",
            "I feel like hell about all of this.",
        ],
    },
    VoiceData {
        mood: Mood::Hostile,
        openers: &[
            "Fine.",
            "You again.",
            "Make it quick.",
            "I don't see why I should, but here.",
            "This is harassment, you know that?",
            "Whatever gets you out of my inbox.",
        ],
        openers_adult: &[
            "Fine.",
            "What the hell do you want now?",
            "Make it quick, I've got a life.",
            "I don't see why I should, but here, damn you.",
            "This is harassment and you bloody well know it.",
            "Whatever gets you the hell out of my inbox.",
        ],
        closers: &[
            "We're done here.",
            "Don't write again.",
            "Now leave me alone.",
            "That's all you're getting.",
            "Go bother somebody else.",
        ],
        closers_adult: &[
            "We're done here.",
            "Piss off.",
            "Now leave me the hell alone.",
            "That's all you're getting, so sod off.",
            "Go bother some other poor bastard.",
        ],
        dodges: &[
            "No comment.",
            "Ask someone who cares.",
            "I've got nothing to say about that.",
            "Not my problem, not my business.",
            "You already asked. The answer hasn't changed.",
        ],
        dodges_adult: &[
            "No comment.",
            "Ask someone who gives a damn.",
            "I've got sod all to say about that.",
            "Not my problem, not my bloody business.",
            "You already asked. The answer hasn't changed, and neither has my patience.",
        ],
        press_true: &[
            "I said what I said. {stmt}",
            "Read it again. {stmt}",
            "Are you deaf? {stmt}",
            "Same as before. {stmt}",
        ],
        press_false: &[
            "If you must. {stmt} Satisfied?",
            "Fine. {stmt} Happy now?",
            "What I said was, well, what I meant was. {stmt}",
            "Don't twist my words. {stmt} There.",
        ],
        press_false_subtle: &[
            "{stmt}",
            "Same answer. {stmt}",
            "As far as I recall, {stmt}",
            "Again. {stmt}",
        ],
        conscience: &[
            "Not that it matters now.",
            "It's not like I wanted any of this.",
            "I'm not proud of how that night went.",
            "Don't look at me like that. I know.",
        ],
        conscience_adult: &[
            "Not that it matters a damn now.",
            "It's not like I wanted any of this crap.",
            "I'm not proud of how that night went, alright?",
            "Don't look at me like that. I bloody know.",
        ],
    },
    VoiceData {
        mood: Mood::Chatty,
        openers: &[
            "Oh, finally someone asks!",
            "I was hoping you'd write.",
            "Oh, you'll want to sit down for this.",
            "Well! Where do I even start.",
            "Ha, I've been telling everyone this all week.",
            "Pull up a chair, I've got thoughts.",
        ],
        openers_adult: &[
            "Oh, finally someone asks!",
            "I was hoping you'd write.",
            "Oh, you'll want to sit down for this, it's a hell of a thing.",
            "Well! Where do I even start.",
            "Ha, I've been telling everyone this all week and nobody listens.",
            "Pull up a chair, I've got thoughts and a bottle open.",
        ],
        closers: &[
            "Anyway, write back soon!",
            "There's more where that came from.",
            "Oh, and ask me about the others, I've got opinions.",
            "Don't be a stranger!",
            "I could go on, and I will if you let me.",
        ],
        closers_adult: &[
            "Anyway, write back soon!",
            "There's more where that came from.",
            "Oh, and ask me about the others, I've got opinions and none of them are kind.",
            "Don't be a stranger!",
            "I could go on, and I will if you let me. Or if you don't.",
        ],
        dodges: &[
            "Oh, I've run out of things to say on that, and that never happens.",
            "You know, on that one I've honestly got nothing. Ask me something else!",
            "Would you believe I've got no idea? I know. Me. Ask something else.",
            "That one's a blank, and I've been racking my brain, truly.",
            "Everything I had on that, you've already got. Which is annoying for both of us.",
        ],
        dodges_adult: &[
            "Oh, I've run out of things to say on that, and that never bloody happens.",
            "You know, on that one I've honestly got nothing. Ask me something else!",
            "Would you believe I've got no idea? I know. Me. Ask something else.",
            "That one's a blank, and I've been racking my brain, truly.",
            "Everything I had on that, you've already got. Which is annoying for both of us.",
        ],
        press_true: &[
            "Word for word, then: {stmt}",
            "Happily. {stmt}",
            "Oh, a hundred times if you like. {stmt}",
            "Still true! {stmt}",
        ],
        press_false: &[
            "Well, more or less. {stmt} Something like that.",
            "Oh, {stmt} Did I say otherwise?",
            "Ha, did I? I meant, well. {stmt} Close enough!",
            "Oh, you're testing me! Fine. {stmt} Roughly.",
        ],
        press_false_subtle: &[
            "{stmt}",
            "Sure. {stmt}",
            "I believe {stmt}",
            "Oh, easily. {stmt}",
        ],
        conscience: &[
            "I just wish I'd said something sooner.",
            "It weighs on me, honestly.",
            "I've not been myself since, you know.",
            "Between you and me, I feel terrible about it all.",
        ],
        conscience_adult: &[
            "I just wish I'd said something sooner.",
            "It weighs on me, honestly, and I'm not one for guilt.",
            "I've not been myself since, you know. Drinking more, talking more.",
            "Between you and me, I feel bloody terrible about it all.",
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
            and press anyone who sounds shaky. Liars drift when pressed; honest people repeat \
            themselves. When you're sure, send me an accusation naming the {noun} and the \
            fact that gives them away.\n\n{chief}",
    nudge: "You're past the halfway mark and the stories are starting to line up. Keep the \
            pressure on, and don't accuse until the grid says so.\n\n{chief}",
    wrong: "That was the wrong {noun}, and now they won't talk to us. It goes on your \
            record. Go back over what you have and try again.\n\n{chief}",
    solved: "That's the one. {culprit} {fact}, and that settles {crime}.\n\nFor the file, \
             here is how it all fit together. {epilogue}\n\nFinal rank: {rank}. Good \
             work.\n\n{chief}",
};

const ADULT: ChiefData = ChiefData {
    intro: "{setting}\n\nOne thing is certain about {crime}: the {noun} responsible {fact}. \
            Find out who the hell that is. Present: {names}. Email them, compare their \
            stories, and lean on anyone who sounds shaky. Liars drift when pressed; honest \
            people just get annoyed. When you're sure, send me an accusation naming the {noun} \
            and the fact that damns them.\n\n{chief}",
    nudge: "You're past halfway and the stories are starting to crack. Don't get comfortable, \
            and don't accuse anyone on a hunch.\n\n{chief}",
    wrong: "Wrong {noun}, and now they've lawyered up. That goes on your record. Go back over \
            what you have and try again, and this time think.\n\n{chief}",
    solved: "Got the bastard. {culprit} {fact}, and that settles {crime}.\n\nFor the file, \
             here's how it all fit together. {epilogue}\n\nFinal rank: {rank}. Now go get a \
             drink.\n\n{chief}",
};

/// Chief templates for the mode.
pub fn chief(adult: bool) -> &'static ChiefData {
    if adult { &ADULT } else { &CLEAN }
}
