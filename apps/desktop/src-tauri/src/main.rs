// Hides the console window on Windows release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tiki_desktop_lib::TikiDesktopApp::run();
}
