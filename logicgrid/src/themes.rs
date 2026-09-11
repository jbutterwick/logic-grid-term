//! Built-in theme bank (A1.5): each theme has 5 categories of 5 items; category 0 is the anchor
//! and at least one category is ordered (its items are listed in ordinal order). Each category
//! carries the verb phrase clues use to describe it ("owns the {}").

/// One category of a theme.
pub(crate) struct ThemeCat {
    pub name: &'static str,
    pub ordered: bool,
    pub items: [&'static str; 5],
    /// "owns the {}"; the anchor uses "is {}".
    pub phrase: &'static str,
    /// Ordered only: "lives on a lower floor than" / "... higher ..." / "live on adjacent floors".
    pub less: &'static str,
    pub more: &'static str,
    pub adjacent: &'static str,
}

/// A theme: title, intro blurb, 5 categories (anchor first, at least one ordered).
pub(crate) struct Theme {
    pub name: &'static str,
    pub intro: &'static str,
    pub cats: [ThemeCat; 5],
}

/// Anchor category: the people (or ghosts) being matched.
const fn a(name: &'static str, items: [&'static str; 5]) -> ThemeCat {
    c(name, "is {}", items)
}

/// Unordered category.
const fn c(name: &'static str, phrase: &'static str, items: [&'static str; 5]) -> ThemeCat {
    ThemeCat {
        name,
        ordered: false,
        items,
        phrase,
        less: "",
        more: "",
        adjacent: "",
    }
}

/// Ordered category.
const fn o(
    name: &'static str,
    phrase: &'static str,
    less: &'static str,
    more: &'static str,
    adjacent: &'static str,
    items: [&'static str; 5],
) -> ThemeCat {
    ThemeCat {
        name,
        ordered: true,
        items,
        phrase,
        less,
        more,
        adjacent,
    }
}

/// All built-in themes.
pub(crate) const THEMES: &[Theme] = &[
    Theme {
        name: "Apartment Block",
        intro: "Five neighbors share a walk-up on Maple Street. Each keeps one pet, wears one \
                color, and won't start the morning without a particular drink. The new mail \
                carrier has it all mixed up.",
        cats: [
            a("Person", ["Alice", "Bob", "Carol", "Dave", "Erin"]),
            c(
                "Pet",
                "owns the {}",
                ["Cat", "Dog", "Fish", "Bird", "Rabbit"],
            ),
            c(
                "Color",
                "wears {}",
                ["Red", "Green", "Blue", "Yellow", "Purple"],
            ),
            o(
                "Floor",
                "lives on the {} floor",
                "lives on a lower floor than",
                "lives on a higher floor than",
                "live on adjacent floors",
                ["1st", "2nd", "3rd", "4th", "5th"],
            ),
            c(
                "Drink",
                "drinks {}",
                ["Tea", "Coffee", "Juice", "Milk", "Water"],
            ),
        ],
    },
    Theme {
        name: "Dinner Party",
        intro: "The dinner party is over and the thank-you notes are due. Who brought which \
                dish, who sat where, who gave what, and who was fashionably late?",
        cats: [
            a("Guest", ["Fiona", "George", "Hannah", "Ivan", "Julia"]),
            o(
                "Arrival",
                "arrived at {}",
                "arrived before",
                "arrived after",
                "arrived half an hour apart",
                ["6:00", "6:30", "7:00", "7:30", "8:00"],
            ),
            c(
                "Dish",
                "brought the {}",
                ["Soup", "Salad", "Pasta", "Curry", "Tacos"],
            ),
            o(
                "Seat",
                "sat in {}",
                "sat to the left of",
                "sat to the right of",
                "sat next to each other",
                ["Seat 1", "Seat 2", "Seat 3", "Seat 4", "Seat 5"],
            ),
            c(
                "Gift",
                "gave {}",
                ["Wine", "Flowers", "Candles", "Chocolate", "Book"],
            ),
        ],
    },
    Theme {
        name: "Marathon",
        intro: "The city marathon results got smudged in the rain. Five runners, five bibs, \
                five pairs of shoes, and only the finish-line photographer's notes to go on.",
        cats: [
            a("Runner", ["Kai", "Lena", "Marco", "Nadia", "Omar"]),
            o(
                "Bib",
                "wore bib {}",
                "wore a lower bib number than",
                "wore a higher bib number than",
                "wore consecutive bib numbers",
                ["#11", "#22", "#33", "#44", "#55"],
            ),
            c(
                "Shoe",
                "ran in {}",
                ["Nike", "Asics", "Brooks", "Hoka", "Saucony"],
            ),
            o(
                "Finish",
                "finished {}",
                "finished ahead of",
                "finished behind",
                "finished one place apart",
                ["1st", "2nd", "3rd", "4th", "5th"],
            ),
            c(
                "City",
                "came from {}",
                ["Boston", "Chicago", "Denver", "Austin", "Seattle"],
            ),
        ],
    },
    Theme {
        name: "Flea Market",
        intro: "Sunday's flea market moved five bargains. The organizer wants to know who \
                bought what, at which stall, for how much, and how they paid.",
        cats: [
            a("Buyer", ["Priya", "Quinn", "Rosa", "Sam", "Tara"]),
            o(
                "Price",
                "paid {}",
                "paid less than",
                "paid more than",
                "paid ten dollars apart",
                ["$10", "$20", "$30", "$40", "$50"],
            ),
            c(
                "Item",
                "bought the {}",
                ["Lamp", "Chair", "Rug", "Mirror", "Vase"],
            ),
            o(
                "Stall",
                "shopped at {}",
                "shopped at a stall left of",
                "shopped at a stall right of",
                "shopped at neighboring stalls",
                ["Stall A", "Stall B", "Stall C", "Stall D", "Stall E"],
            ),
            c(
                "Payment",
                "paid with {}",
                ["Cash", "Card", "Check", "App", "Trade"],
            ),
        ],
    },
    Theme {
        name: "Cul-de-Sac",
        intro: "Five houses ring the cul-de-sac, and the newest arrival is trying to keep the \
                neighbors straight: who drives what, who is how old, and who keeps inviting \
                everyone to chess night.",
        cats: [
            a("Neighbor", ["Uma", "Victor", "Wendy", "Xavier", "Yara"]),
            o(
                "House",
                "lives at {}",
                "lives at a lower number than",
                "lives at a higher number than",
                "live next door to each other",
                ["No. 1", "No. 3", "No. 5", "No. 7", "No. 9"],
            ),
            c(
                "Car",
                "drives the {}",
                ["Sedan", "Truck", "Wagon", "Coupe", "Van"],
            ),
            o(
                "Age",
                "is {} years old",
                "is younger than",
                "is older than",
                "are five years apart in age",
                ["25", "30", "35", "40", "45"],
            ),
            c(
                "Hobby",
                "is into {}",
                ["Chess", "Golf", "Piano", "Yoga", "Baking"],
            ),
        ],
    },
    Theme {
        name: "Harbor",
        intro: "Five boats are tied up in the harbor and the harbormaster's log is a mess. \
                Sort out who sails what, from which dock, under which flag, and when they \
                leave.",
        cats: [
            a("Sailor", ["Zane", "Ada", "Bruno", "Cleo", "Dmitri"]),
            c(
                "Boat",
                "sails the {}",
                ["Sloop", "Ketch", "Yawl", "Cutter", "Dinghy"],
            ),
            o(
                "Dock",
                "is tied up at {}",
                "is at a lower dock number than",
                "is at a higher dock number than",
                "are at neighboring docks",
                ["Dock 1", "Dock 2", "Dock 3", "Dock 4", "Dock 5"],
            ),
            c(
                "Flag",
                "flies the {} flag",
                ["White", "Black", "Orange", "Teal", "Gold"],
            ),
            o(
                "Departure",
                "departs on {}",
                "departs earlier in the week than",
                "departs later in the week than",
                "depart on consecutive days",
                ["Mon", "Tue", "Wed", "Thu", "Fri"],
            ),
        ],
    },
    Theme {
        name: "High School",
        intro: "The yearbook committee lost its notes. Five students, their best subjects, \
                grades, lockers, and clubs need matching before the print deadline.",
        cats: [
            a("Student", ["Elif", "Femi", "Greta", "Hiro", "Ines"]),
            c(
                "Subject",
                "is best at {}",
                ["Math", "History", "Art", "Biology", "Music"],
            ),
            o(
                "Grade",
                "is in {}",
                "is in a lower grade than",
                "is in a higher grade than",
                "are one grade apart",
                ["Grade 8", "Grade 9", "Grade 10", "Grade 11", "Grade 12"],
            ),
            o(
                "Locker",
                "has {}",
                "has a lower locker number than",
                "has a higher locker number than",
                "have lockers side by side",
                [
                    "Locker 101",
                    "Locker 102",
                    "Locker 103",
                    "Locker 104",
                    "Locker 105",
                ],
            ),
            c(
                "Club",
                "belongs to the {} club",
                ["Robotics", "Drama", "Debate", "Soccer", "Choir"],
            ),
        ],
    },
    Theme {
        name: "Heist Crew",
        intro: "The vault job went off without a hitch, but the planner shredded the roster. \
                Reconstruct who did what, when they went in, how they got out, and what cut \
                they took.",
        cats: [
            a("Thief", ["Ace", "Blaze", "Cinder", "Dutch", "Echo"]),
            c(
                "Specialty",
                "handles the {}",
                ["Safe", "Wheels", "Lookout", "Hacking", "Disguises"],
            ),
            o(
                "Cut",
                "takes a {} cut",
                "takes a smaller cut than",
                "takes a bigger cut than",
                "have cuts five percent apart",
                ["10%", "15%", "20%", "25%", "30%"],
            ),
            c(
                "Getaway",
                "escapes by {}",
                ["Taxi", "Sewer", "Rooftop", "Subway", "Boat"],
            ),
            o(
                "Entry",
                "goes in at {}",
                "goes in before",
                "goes in after",
                "go in fifteen minutes apart",
                ["9:00", "9:15", "9:30", "9:45", "10:00"],
            ),
        ],
    },
    Theme {
        name: "Haunted Hotel",
        intro: "The Grand Hotel's night manager keeps a ledger of its five permanent \
                residents, and a draft blew the pages apart. Which ghost haunts which room, \
                what noise it makes, when it died, and what it clings to?",
        cats: [
            a(
                "Ghost",
                ["Agnes", "Barnaby", "Cordelia", "Desmond", "Elspeth"],
            ),
            o(
                "Room",
                "haunts {}",
                "haunts a lower floor than",
                "haunts a higher floor than",
                "haunt rooms one floor apart",
                ["Room 101", "Room 202", "Room 303", "Room 404", "Room 505"],
            ),
            c(
                "Sound",
                "is known for {}",
                ["Moaning", "Rattling", "Whispers", "Footsteps", "Laughter"],
            ),
            o(
                "Died",
                "died in {}",
                "died before",
                "died after",
                "died twenty years apart",
                ["1840", "1860", "1880", "1900", "1920"],
            ),
            c(
                "Relic",
                "is bound to the {}",
                ["Locket", "Candle", "Mirror", "Key", "Portrait"],
            ),
        ],
    },
    Theme {
        name: "Space Station",
        intro: "Orbital Station Kestrel rotates its five crew through decks, jobs, and snack \
                lockers. Mission control needs the roster straight before the next supply run.",
        cats: [
            a("Crew", ["Nova", "Orin", "Petra", "Quill", "Ravi"]),
            o(
                "Deck",
                "works on {}",
                "works on a lower deck than",
                "works on a higher deck than",
                "work on adjacent decks",
                ["Deck 1", "Deck 2", "Deck 3", "Deck 4", "Deck 5"],
            ),
            c(
                "Job",
                "is the {}",
                ["Pilot", "Medic", "Engineer", "Botanist", "Comms"],
            ),
            o(
                "Arrived",
                "arrived in {}",
                "arrived before",
                "arrived after",
                "arrived a month apart",
                ["March", "April", "May", "June", "July"],
            ),
            c(
                "Snack",
                "snacks on {}",
                ["Pudding", "Noodles", "Crackers", "Jerky", "Tang"],
            ),
        ],
    },
    Theme {
        name: "Bake-Off",
        intro: "The village bake-off judges have eaten everything and remembered nothing. \
                Match each baker to their bake, their oven, their score, and their secret \
                ingredient.",
        cats: [
            a("Baker", ["Hazel", "Idris", "June", "Kofi", "Luz"]),
            c(
                "Bake",
                "made the {}",
                ["Scones", "Tart", "Sourdough", "Eclairs", "Pavlova"],
            ),
            o(
                "Oven",
                "used {}",
                "used a lower-numbered oven than",
                "used a higher-numbered oven than",
                "used neighboring ovens",
                ["Oven 1", "Oven 2", "Oven 3", "Oven 4", "Oven 5"],
            ),
            o(
                "Score",
                "scored {}",
                "scored lower than",
                "scored higher than",
                "scored one point apart",
                ["6/10", "7/10", "8/10", "9/10", "10/10"],
            ),
            c(
                "Secret",
                "swears by {}",
                ["Cardamom", "Rum", "Lemon", "Miso", "Espresso"],
            ),
        ],
    },
    Theme {
        name: "Band Tour",
        intro: "The band's tour manager quit mid-tour and took the notes. Who plays what, who \
                sleeps where on the bus, what they demand backstage, and who joined when?",
        cats: [
            a("Musician", ["Mae", "Nico", "Ola", "Pip", "Rex"]),
            c(
                "Instrument",
                "plays {}",
                ["Bass", "Drums", "Keys", "Guitar", "Vocals"],
            ),
            o(
                "Bunk",
                "sleeps in {}",
                "sleeps in a lower bunk than",
                "sleeps in a higher bunk than",
                "sleep in neighboring bunks",
                ["Bunk 1", "Bunk 2", "Bunk 3", "Bunk 4", "Bunk 5"],
            ),
            c(
                "Rider",
                "demands {}",
                ["Gummies", "Towels", "Kombucha", "Pickles", "Incense"],
            ),
            o(
                "Joined",
                "joined in {}",
                "joined before",
                "joined after",
                "joined two years apart",
                ["2015", "2017", "2019", "2021", "2023"],
            ),
        ],
    },
    Theme {
        name: "Garden Club",
        intro: "The allotment committee is settling a dispute over plots, plants, and pests. \
                Five gardeners, five plots, and everyone claims to be up first with the hose.",
        cats: [
            a("Gardener", ["Sol", "Tamsin", "Ulf", "Vera", "Wes"]),
            c(
                "Plant",
                "grows {}",
                ["Roses", "Tomatoes", "Dahlias", "Pumpkins", "Mint"],
            ),
            o(
                "Plot",
                "tends {}",
                "tends a lower-numbered plot than",
                "tends a higher-numbered plot than",
                "tend neighboring plots",
                ["Plot 1", "Plot 2", "Plot 3", "Plot 4", "Plot 5"],
            ),
            c(
                "Pest",
                "battles {}",
                ["Slugs", "Aphids", "Rabbits", "Crows", "Moles"],
            ),
            o(
                "Watering",
                "waters at {}",
                "waters earlier than",
                "waters later than",
                "water an hour apart",
                ["6am", "7am", "8am", "9am", "10am"],
            ),
        ],
    },
    Theme {
        name: "Manor Mystery",
        intro: "Someone let the peacocks loose at Thornfield Manor during the storm. Five \
                guests, five rooms, five alibis. The constable would like it sorted before \
                breakfast.",
        cats: [
            a(
                "Guest",
                ["Ashford", "Blythe", "Crane", "Dimbleby", "Everly"],
            ),
            c(
                "Room",
                "was in the {}",
                ["Library", "Kitchen", "Study", "Ballroom", "Cellar"],
            ),
            c(
                "Alibi",
                "claims to have been {}",
                ["Reading", "Cooking", "Sleeping", "Dancing", "Praying"],
            ),
            o(
                "Last Seen",
                "was last seen at {}",
                "was last seen before",
                "was last seen after",
                "were last seen an hour apart",
                ["9pm", "10pm", "11pm", "Midnight", "1am"],
            ),
            c(
                "Motive",
                "is driven by {}",
                ["Debt", "Jealousy", "Revenge", "Greed", "Blackmail"],
            ),
        ],
    },
    Theme {
        name: "Coffee Shop",
        intro: "The new barista at Grind House is trying to memorize the morning regulars: \
                their orders, their tables, their pastries, and exactly when each one walks in.",
        cats: [
            a("Regular", ["Ana", "Ben", "Cy", "Dee", "Eli"]),
            c(
                "Order",
                "orders the {}",
                ["Latte", "Espresso", "Mocha", "Chai", "Cortado"],
            ),
            o(
                "Table",
                "sits at {}",
                "sits at a lower-numbered table than",
                "sits at a higher-numbered table than",
                "sit at neighboring tables",
                ["Table 1", "Table 2", "Table 3", "Table 4", "Table 5"],
            ),
            o(
                "Arrives",
                "shows up at {}",
                "shows up before",
                "shows up after",
                "show up half an hour apart",
                ["7:00", "7:30", "8:00", "8:30", "9:00"],
            ),
            c(
                "Pastry",
                "always gets the {}",
                ["Croissant", "Bagel", "Muffin", "Danish", "Biscotti"],
            ),
        ],
    },
];

#[cfg(test)]
mod tests {
    use super::THEMES;

    /// Must stay in sync with `MAX_LABEL` in logictui's play screen, or labels truncate.
    #[test]
    fn labels_fit_max_label() {
        for t in THEMES {
            for c in &t.cats {
                for s in std::iter::once(c.name).chain(c.items) {
                    assert!(s.chars().count() <= 10, "{s:?} longer than MAX_LABEL");
                }
            }
        }
    }

    #[test]
    fn themes_are_well_formed() {
        for t in THEMES {
            assert!(!t.intro.is_empty(), "{}", t.name);
            assert!(
                t.cats.iter().any(|c| c.ordered),
                "{} has no ordered cat",
                t.name
            );
            for c in &t.cats {
                assert!(c.phrase.contains("{}"), "{}/{}", t.name, c.name);
                assert_eq!(
                    c.ordered,
                    !c.less.is_empty() && !c.more.is_empty() && !c.adjacent.is_empty(),
                    "{}/{}",
                    t.name,
                    c.name
                );
            }
        }
    }
}
