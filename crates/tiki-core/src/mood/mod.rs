//! Tiki's mood: the mascot's face summarizes the user's day at a glance.

mod mood_kind;
mod mood_resolver;
mod mood_signals;

pub use mood_kind::Mood;
pub use mood_resolver::MoodResolver;
pub use mood_signals::MoodSignals;
