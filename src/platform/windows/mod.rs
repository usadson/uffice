// Copyright (C) 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use windows::{
    core::PCWSTR,
    w,
    Win32::{
        Foundation::{HWND, GetLastError},
        UI::{
            Shell::ShellExecuteW,
            WindowsAndMessaging::SW_SHOWNORMAL,
        },
        System::Threading::{
            GetCurrentThread,
            SetThreadDescription,
        },
    },
};

pub mod registry;

const OPEN_VERB: PCWSTR = w!("open");

pub fn open_file_user(path: &str) {
    println!("Path: {}", path);
    let path: Vec<u16> = path.encode_utf16().collect();
    let window: HWND = Default::default();
    println!("pre: {:?}", unsafe { GetLastError() });
    let result = unsafe {
        ShellExecuteW(window, OPEN_VERB, PCWSTR(path.as_ptr()), None, None, SW_SHOWNORMAL)
    }.0;
    println!("open_file_user: {:?}, {}", unsafe { GetLastError() }, result);

    // use self::registry::{
    //     RegistryKey,
    //     RegistryError,
    //     PredefinedRegistryKey,
    // };

    // _ = (|| -> Result<(), RegistryError> {
    //     let classes_root = RegistryKey::open(PredefinedRegistryKey::ClassesRoot)?;
    //     let subkey = classes_root.open_subkey(".docx")?;

    //     for value in subkey.values()? {
    //         if value.name.is_empty() {
    //             let subkey_name = format!("{}\\shell\\Open\\command", value.data.as_str()?);
    //             let key = classes_root.open_subkey(&subkey_name)?;
    //             key.values().unwrap().first().unwrap()
    //         }
    //     }

    //     Ok(())
    // })()
}

pub fn set_current_thread_name(name: &str) {
    let name: Vec<u16> = name.encode_utf16().collect();
    unsafe {
        _ = SetThreadDescription(GetCurrentThread(), PCWSTR(name.as_ptr()));
    }
}
