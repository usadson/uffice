// Copyright (C) 2022 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

#[cfg(windows)]
pub mod windows;

#[cfg(windows)]
pub use self::windows as implementation;

pub fn open_file_user(path: &str) {
    implementation::open_file_user(path);
}

pub fn set_current_thread_name(name: &str) {
    implementation::set_current_thread_name(name);
}
