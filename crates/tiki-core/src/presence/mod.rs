//! Signs, read from the operating system, that the user is busy elsewhere, such as a call,
//! so Tiki keeps quiet meanwhile.

mod call_detector;
mod consent_store;

pub use call_detector::CallDetector;
pub use consent_store::ConsentStore;
