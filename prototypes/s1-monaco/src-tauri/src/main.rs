// Prevents an additional console window on Windows in release.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    diffy_s1_monaco_lib::run();
}
