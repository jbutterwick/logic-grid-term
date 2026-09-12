//! Casefile: a narrative deduction game built on `logicgrid`. See REQUIREMENTS.md section D.
//!
//! A [`Case`] wraps a generated puzzle: the anchor items become suspects, one non-anchor entity
//! becomes the guilty fact, and the puzzle's minimal clue set is handed out as truthful
//! statements (liars add false ones on top). A [`Game`] runs the email inbox around it. Nothing
//! here touches the filesystem, clock, or OS randomness, so it builds for wasm.

#![deny(missing_docs)]

mod case;
mod content;
mod game;
mod text;
pub mod ui;
#[cfg(target_arch = "wasm32")]
pub mod web;

pub use case::{Case, Frame, Level, Mood, Settings, Statement, Suspect};
pub use content::{ChiefData, FrameData, VoiceData, chief, frame, voice};
pub use game::{Email, Game, Question, Rank};
pub use logicgrid::{Cat, Clue, Entity, Grid, Item, Mark, Puzzle};
pub use text::{first_person, render};

/// Names of the built-in themes, indexable by [`Settings::theme`].
pub fn theme_names() -> Vec<&'static str> {
    logicgrid::theme_names().collect()
}
