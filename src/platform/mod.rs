// Copyright (C) 2022 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(windows)]
pub mod windows;

#[cfg(target_os = "macos")]
pub use self::macos as implementation;

#[cfg(windows)]
pub use self::windows as implementation;

pub fn show_message_box_blocking(title: &str, message: &str) {
    implementation::show_message_box_blocking(title, message);
}

pub fn open_file_user(path: &str) {
    implementation::open_file_user(path);
}

pub fn set_current_thread_name(name: &str) {
    implementation::set_current_thread_name(name);
}

/// Saves the current state in case that the application crashes or the system
/// is rebooted automatically.
pub fn save_restore_arguments(arguments: crate::CommandLineArguments) {
    implementation::save_restore_arguments(arguments)
}
