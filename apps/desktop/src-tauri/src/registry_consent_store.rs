use std::io;

use tiki_core::presence::ConsentStore;
use winreg::RegKey;
use winreg::enums::HKEY_CURRENT_USER;

/// Windows' record of the apps that used the microphone, in the user's registry.
pub struct RegistryConsentStore;

impl RegistryConsentStore {
    const MICROPHONE_STORE: &'static str = r"Software\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\microphone";
    const LAST_USED_TIME_STOP: &'static str = "LastUsedTimeStop";
    const KEY_SEPARATOR: &'static str = r"\";

    /// The key `key` below the store, or `None` when it does not exist.
    fn open(key: &[&str]) -> io::Result<Option<RegKey>> {
        let path = std::iter::once(Self::MICROPHONE_STORE)
            .chain(key.iter().copied())
            .collect::<Vec<_>>()
            .join(Self::KEY_SEPARATOR);
        Self::unless_missing(RegKey::predef(HKEY_CURRENT_USER).open_subkey(path))
    }

    /// A key or value that does not exist is not an error: the app never used the microphone.
    fn unless_missing<T>(read: io::Result<T>) -> io::Result<Option<T>> {
        match read {
            Ok(found) => Ok(Some(found)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error),
        }
    }
}

impl ConsentStore for RegistryConsentStore {
    fn subkey_names(&self, key: &[&str]) -> io::Result<Vec<String>> {
        match Self::open(key)? {
            Some(opened) => opened.enum_keys().collect(),
            None => Ok(Vec::new()),
        }
    }

    fn last_used_time_stop(&self, key: &[&str]) -> io::Result<Option<u64>> {
        match Self::open(key)? {
            Some(opened) => Self::unless_missing(opened.get_value(Self::LAST_USED_TIME_STOP)),
            None => Ok(None),
        }
    }
}
