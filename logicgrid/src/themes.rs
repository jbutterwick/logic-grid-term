//! Built-in theme bank (A1.5): each theme has 5 categories of 5 items; category 0 is the anchor
//! and at least one category is ordered (its items are listed in ordinal order).

/// One category of a theme: display name, whether it is ordered, and 5 items.
pub(crate) struct ThemeCat {
    pub name: &'static str,
    pub ordered: bool,
    pub items: [&'static str; 5],
}

/// A theme: 5 categories, anchor first, at least one ordered.
pub(crate) type Theme = [ThemeCat; 5];

const fn c(name: &'static str, ordered: bool, items: [&'static str; 5]) -> ThemeCat {
    ThemeCat {
        name,
        ordered,
        items,
    }
}

/// All built-in themes.
pub(crate) const THEMES: &[Theme] = &[
    [
        c("Person", false, ["Alice", "Bob", "Carol", "Dave", "Erin"]),
        c("Pet", false, ["Cat", "Dog", "Fish", "Bird", "Rabbit"]),
        c("Color", false, ["Red", "Green", "Blue", "Yellow", "Purple"]),
        c("Floor", true, ["1st", "2nd", "3rd", "4th", "5th"]),
        c("Drink", false, ["Tea", "Coffee", "Juice", "Milk", "Water"]),
    ],
    [
        c(
            "Guest",
            false,
            ["Fiona", "George", "Hannah", "Ivan", "Julia"],
        ),
        c("Arrival", true, ["6:00", "6:30", "7:00", "7:30", "8:00"]),
        c("Dish", false, ["Soup", "Salad", "Pasta", "Curry", "Tacos"]),
        c(
            "Seat",
            true,
            ["Seat 1", "Seat 2", "Seat 3", "Seat 4", "Seat 5"],
        ),
        c(
            "Gift",
            false,
            ["Wine", "Flowers", "Candles", "Chocolate", "Book"],
        ),
    ],
    [
        c("Runner", false, ["Kai", "Lena", "Marco", "Nadia", "Omar"]),
        c("Bib", true, ["#11", "#22", "#33", "#44", "#55"]),
        c(
            "Shoe",
            false,
            ["Nike", "Asics", "Brooks", "Hoka", "Saucony"],
        ),
        c("Finish", true, ["1st", "2nd", "3rd", "4th", "5th"]),
        c(
            "City",
            false,
            ["Boston", "Chicago", "Denver", "Austin", "Seattle"],
        ),
    ],
    [
        c("Buyer", false, ["Priya", "Quinn", "Rosa", "Sam", "Tara"]),
        c("Price", true, ["$10", "$20", "$30", "$40", "$50"]),
        c("Item", false, ["Lamp", "Chair", "Rug", "Mirror", "Vase"]),
        c(
            "Stall",
            true,
            ["Stall A", "Stall B", "Stall C", "Stall D", "Stall E"],
        ),
        c("Payment", false, ["Cash", "Card", "Check", "App", "Trade"]),
    ],
    [
        c(
            "Neighbor",
            false,
            ["Uma", "Victor", "Wendy", "Xavier", "Yara"],
        ),
        c("House", true, ["No. 1", "No. 3", "No. 5", "No. 7", "No. 9"]),
        c("Car", false, ["Sedan", "Truck", "Wagon", "Coupe", "Van"]),
        c("Age", true, ["25", "30", "35", "40", "45"]),
        c("Hobby", false, ["Chess", "Golf", "Piano", "Yoga", "Baking"]),
    ],
    [
        c("Sailor", false, ["Zane", "Ada", "Bruno", "Cleo", "Dmitri"]),
        c(
            "Boat",
            false,
            ["Sloop", "Ketch", "Yawl", "Cutter", "Dinghy"],
        ),
        c(
            "Dock",
            true,
            ["Dock 1", "Dock 2", "Dock 3", "Dock 4", "Dock 5"],
        ),
        c("Flag", false, ["White", "Black", "Orange", "Teal", "Gold"]),
        c("Departure", true, ["Mon", "Tue", "Wed", "Thu", "Fri"]),
    ],
    [
        c("Student", false, ["Elif", "Femi", "Greta", "Hiro", "Ines"]),
        c(
            "Subject",
            false,
            ["Math", "History", "Art", "Biology", "Music"],
        ),
        c(
            "Grade",
            true,
            ["Grade 8", "Grade 9", "Grade 10", "Grade 11", "Grade 12"],
        ),
        c(
            "Locker",
            true,
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
            false,
            ["Robotics", "Drama", "Debate", "Soccer", "Choir"],
        ),
    ],
];
