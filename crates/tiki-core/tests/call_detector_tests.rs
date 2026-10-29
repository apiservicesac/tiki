use std::collections::HashMap;
use std::io;
use std::path::Path;

use tiki_core::presence::{CallDetector, ConsentStore};

const DESKTOP_APPS: &str = "NonPackaged";
const TIKI: &str = r"C:\Program Files\Tiki\tiki.exe";
const TEAMS_KEY: &str = "MSTeams_8wekyb3d8bbwe";
const ZOOM_KEY: &str = "C:#Users#ana#AppData#Roaming#Zoom#bin#Zoom.exe";
const TIKI_KEY: &str = "C:#Program Files#Tiki#tiki.exe";
/// A `LastUsedTimeStop` of an app that let the microphone go.
const STOPPED: u64 = 134_060_000_000_000_000;
const IN_USE: u64 = 0;

/// The microphone consent store as Windows would hold it, in memory.
#[derive(Default)]
struct FakeConsentStore {
    subkeys: HashMap<Vec<String>, Vec<String>>,
    stops: HashMap<Vec<String>, u64>,
}

impl FakeConsentStore {
    fn with_store_app(mut self, name: &str, stop: Option<u64>) -> Self {
        self.add(&[], name, stop);
        self
    }

    fn with_desktop_app(mut self, name: &str, stop: Option<u64>) -> Self {
        if !self
            .subkey_names(&[])
            .unwrap()
            .iter()
            .any(|app| app == DESKTOP_APPS)
        {
            self.add(&[], DESKTOP_APPS, None);
        }
        self.add(&[DESKTOP_APPS], name, stop);
        self
    }

    fn add(&mut self, parent: &[&str], name: &str, stop: Option<u64>) {
        self.subkeys
            .entry(Self::path(parent))
            .or_default()
            .push(name.to_owned());
        if let Some(stop) = stop {
            let mut key = Self::path(parent);
            key.push(name.to_owned());
            self.stops.insert(key, stop);
        }
    }

    fn path(key: &[&str]) -> Vec<String> {
        key.iter().map(|name| (*name).to_owned()).collect()
    }
}

impl ConsentStore for FakeConsentStore {
    fn subkey_names(&self, key: &[&str]) -> io::Result<Vec<String>> {
        Ok(self
            .subkeys
            .get(&Self::path(key))
            .cloned()
            .unwrap_or_default())
    }

    fn last_used_time_stop(&self, key: &[&str]) -> io::Result<Option<u64>> {
        Ok(self.stops.get(&Self::path(key)).copied())
    }
}

fn in_call(store: FakeConsentStore) -> bool {
    CallDetector::new(store, Path::new(TIKI))
        .another_app_uses_microphone()
        .unwrap()
}

#[test]
fn nobody_ever_used_the_microphone() {
    assert!(!in_call(FakeConsentStore::default()));
}

#[test]
fn a_store_app_holding_the_microphone_is_a_call() {
    assert!(in_call(
        FakeConsentStore::default().with_store_app(TEAMS_KEY, Some(IN_USE))
    ));
}

#[test]
fn a_desktop_app_holding_the_microphone_is_a_call() {
    assert!(in_call(
        FakeConsentStore::default()
            .with_store_app(TEAMS_KEY, Some(STOPPED))
            .with_desktop_app(ZOOM_KEY, Some(IN_USE))
    ));
}

#[test]
fn apps_that_let_the_microphone_go_are_not_a_call() {
    assert!(!in_call(
        FakeConsentStore::default()
            .with_store_app(TEAMS_KEY, Some(STOPPED))
            .with_desktop_app(ZOOM_KEY, Some(STOPPED))
    ));
}

#[test]
fn an_app_without_a_stop_time_is_not_a_call() {
    assert!(!in_call(
        FakeConsentStore::default().with_desktop_app(ZOOM_KEY, None)
    ));
}

#[test]
fn tiki_using_the_microphone_itself_is_not_a_call() {
    assert!(!in_call(
        FakeConsentStore::default().with_desktop_app(TIKI_KEY, Some(IN_USE))
    ));
    assert!(!in_call(
        FakeConsentStore::default().with_desktop_app(&TIKI_KEY.to_uppercase(), Some(IN_USE))
    ));
}
