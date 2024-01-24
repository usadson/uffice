// Copyright (C) 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

pub fn open_file_user(path: &str) {
}

pub fn set_current_thread_name(name: &str) {
}

pub unsafe fn load_symbol(library_name: &str, symbol_name: &str) -> Option<unsafe extern "system" fn() -> isize> {
    unimplemented!()
}

pub fn save_restore_arguments(arguments: crate::CommandLineArguments) {
}

pub fn show_message_box_blocking(title: &str, message: &str) {
    unimplemented!()
}
