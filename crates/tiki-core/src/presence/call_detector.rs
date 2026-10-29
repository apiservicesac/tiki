use std::io;
use std::path::Path;

use super::ConsentStore;

/// Tells whether another app holds the microphone, which Tiki takes as a call in progress.
///
/// Windows keeps one key per app that used the microphone: Store apps right under the
/// store, desktop apps under `NonPackaged`, named by their executable path with `#` for
/// each `\`. While an app holds the microphone its `LastUsedTimeStop` is 0. Tiki's own use
/// never counts.
pub struct CallDetector<Store: ConsentStore> {
    store: Store,
    /// Tiki's own executable, written the way the store names desktop apps.
    own_app_key: String,
}

impl<Store: ConsentStore> CallDetector<Store> {
    /// The key that holds the desktop apps, which are not installed from the Store.
    const DESKTOP_APPS_KEY: &'static str = "NonPackaged";
    /// The separator of an executable path, and what the store writes in its place.
    const PATH_SEPARATOR: char = '\\';
    const KEY_PATH_SEPARATOR: &'static str = "#";
    /// `LastUsedTimeStop` of an app that has not stopped using the microphone.
    const STILL_IN_USE: u64 = 0;

    pub fn new(store: Store, own_executable: &Path) -> Self {
        Self {
            store,
            own_app_key: own_executable
                .to_string_lossy()
                .replace(Self::PATH_SEPARATOR, Self::KEY_PATH_SEPARATOR),
        }
    }

    pub fn another_app_uses_microphone(&self) -> io::Result<bool> {
        for app in self.store.subkey_names(&[])? {
            let in_use = if app == Self::DESKTOP_APPS_KEY {
                self.a_desktop_app_uses_microphone()?
            } else {
                self.uses_microphone(&[&app])?
            };
            if in_use {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn a_desktop_app_uses_microphone(&self) -> io::Result<bool> {
        for app in self.store.subkey_names(&[Self::DESKTOP_APPS_KEY])? {
            if !app.eq_ignore_ascii_case(&self.own_app_key)
                && self.uses_microphone(&[Self::DESKTOP_APPS_KEY, &app])?
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn uses_microphone(&self, key: &[&str]) -> io::Result<bool> {
        Ok(self.store.last_used_time_stop(key)? == Some(Self::STILL_IN_USE))
    }
}
