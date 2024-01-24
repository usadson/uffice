// Copyright (C) 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use windows::{
    core::{
        PCSTR,
        PCWSTR,
        HRESULT,
    },
    w,
    Win32::{
        Foundation::{
            GetLastError,
            HANDLE,
            HWND,
        },
        UI::{
            Shell::ShellExecuteW,
            WindowsAndMessaging::{
                MB_ICONERROR,
                MB_OK,
                MessageBoxA,
                SW_SHOWNORMAL,
            },
        },
        System::LibraryLoader::{
            GetProcAddress,
            LoadLibraryA,
        },
        System::{
            Threading::GetCurrentThread,
            Recovery::RegisterApplicationRestart,
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

pub unsafe fn load_symbol(library_name: &str, symbol_name: &str) -> Option<unsafe extern "system" fn() -> isize> {
    let Ok(kernel) = LoadLibraryA(PCSTR(library_name.as_ptr())) else {
        return None;
    };

    GetProcAddress(kernel, PCSTR(symbol_name.as_ptr()))
}

pub fn set_current_thread_name(name: &str) {
    let name: Vec<u16> = name.encode_utf16().collect();
    type FuncType = unsafe extern "system" fn(hthread: HANDLE, lpthreaddescription: PCWSTR) -> HRESULT;

    unsafe {
        if let Some(func) = load_symbol("Kernel32.dll", "SetThreadDescription") {
            let func: FuncType = std::mem::transmute(func);
            _ = func(GetCurrentThread(), PCWSTR(name.as_ptr()));
        }
    }
}

/// Saves the current state in case that the application crashes or the system
/// is rebooted automatically.
///
/// Warning: This function does not compile for Windows versions prior to Vista.
pub fn save_restore_arguments(arguments: crate::CommandLineArguments) {
    let args: Vec<u16> = arguments.files
        .join(" ")
        .encode_utf16()
        .collect();

    let result = unsafe {
        RegisterApplicationRestart(PCWSTR(args.as_ptr()), Default::default())
    };

    println!("Saving state: {:?}", result);

    #[cfg(debug_assertions)]
    if let Err(err) = result {
        println!("[Win32] Failed to register application restart: {:?}", err);
    }
}

pub fn show_message_box_blocking(title: &str, message: &str) {
    unsafe {
        MessageBoxA(None, windows::core::PCSTR(message.as_ptr()), windows::core::PCSTR(title.as_ptr()), MB_ICONERROR | MB_OK);
    }
}
