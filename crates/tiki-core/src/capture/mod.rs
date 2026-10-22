//! Quick capture: a task the user confirmed, with the fields the assistant filled in or the
//! user chose. Tiki never guesses fields with word rules: the AI reads the sentence, or the
//! user picks them.

mod confirmed_capture;

pub use confirmed_capture::ConfirmedCapture;
