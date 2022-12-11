// Copyright (C) 2022 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use uffice_lib::constants;

#[cfg(target_os = "windows")]
use registry::{Hive, Security};

#[cfg(target_os = "windows")]
fn create_associations() {
    let reg_key = Hive::CurrentUser.open("Software\\Microsoft\\Windows\\CurrentVersion\\App Paths", Security::CreateSubKey)
            .unwrap();

    let exe_path = std::env::current_exe().unwrap();
    
    reg_key.set_value("uffice.exe", &registry::value::Data::String(utfx::U16CString::from_str(exe_path.to_string_lossy()).unwrap()))
        .expect("Failed to set App Path");
}

#[cfg(target_os = "windows")]
pub fn add_file_associations() {
    match Hive::ClassesRoot.open(constants::component::document::PROG_ID, Security::Write) {
        Err(error) => match error {
            registry::key::Error::NotFound(_, _) => {
                create_associations();
            }
            registry::key::Error::PermissionDenied(description, err) => {
                panic!("[File] [Associations]Permission Denied {} {:?}", description, err);
            }
            registry::key::Error::InvalidNul(_) => panic!("Invalid build: registry PROG_ID is invalid!"),
            registry::key::Error::Unknown(description, err) => {
                panic!("[File] [Associations] Unknown Error {} {:?}", description, err)
            }
            _ => todo!(),
        }
        _ => ()
    }
}

#[cfg(not(target_os = "windows"))]
pub fn add_file_associations() {
    // TODO Support more platforms :)
}
