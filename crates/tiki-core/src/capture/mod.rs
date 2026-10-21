//! Quick capture: turns a sentence typed or spoken by the user into a task draft.

mod capture_draft;
mod capture_intent;
mod capture_intent_reader;
mod confirmed_capture;
mod date_phrase_matcher;
mod quick_capture_parser;

pub use capture_draft::CaptureDraft;
pub use capture_intent::{CaptureIntent, CaptureOutcome};
pub use capture_intent_reader::CaptureIntentReader;
pub use confirmed_capture::ConfirmedCapture;
pub use date_phrase_matcher::DatePhraseMatcher;
pub use quick_capture_parser::QuickCaptureParser;
