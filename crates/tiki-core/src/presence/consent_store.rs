use std::io;

/// Windows' record of the apps that asked for the microphone (the `CapabilityAccessManager`
/// consent store), read through this trait so the rules can be tested without Windows.
///
/// A key is the list of names leading to it from the store: `&[]` is the store itself.
/// A key that does not exist reads as empty.
pub trait ConsentStore {
    /// The names of the keys right under `key`.
    fn subkey_names(&self, key: &[&str]) -> io::Result<Vec<String>>;

    /// When the app of `key` last stopped using the microphone; `Some(0)` while it still
    /// does, `None` when the key has no such value.
    fn last_used_time_stop(&self, key: &[&str]) -> io::Result<Option<u64>>;
}
