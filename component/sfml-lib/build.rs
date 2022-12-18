// Copyright (C) 2022 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use std::env;

// Keep up-to-date!
const SFML_VERSION: &str = "2.5.1";

const SFML_BASE_URL: &str = "https://www.sfml-dev.org/files/SFML";

// Windows: https://www.sfml-dev.org/files/SFML-2.5.1-windows-vc15-64-bit.zip
// Linux:   https://www.sfml-dev.org/files/SFML-2.5.1-linux-gcc-64-bit.tar.gz

fn main() {
    download_precompiled_sfml();
}

#[cfg(target_env = "msvc")]
fn download_precompiled_sfml() {
    use std::{path::Path, io::{Cursor, ErrorKind}};

    use http_req::response::StatusCode;

    let exe_directory = format!("{}\\..\\..\\target\\{}", env!("CARGO_MANIFEST_DIR"), env::var("PROFILE").unwrap());
    let out_dir = format!("{}\\SFML", exe_directory);

    println!("exe_directory = {}", exe_directory);
    println!("out_dir = {}", out_dir);

    let mut download = true;

    if let Err(e) = std::fs::create_dir_all(Path::new(&out_dir)) {
        if e.kind() == ErrorKind::AlreadyExists {
            download = false;
        } else {
            panic!("Error \"{}\": {:?}", out_dir, e);
        }
    }

    if download {
        let mut response_body = Vec::new();
        let response = http_req::request::get(format!("{}-{}-windows-vc15-64-bit.zip", SFML_BASE_URL, SFML_VERSION), &mut response_body)
                .expect("Failed to download SFML");

        assert!(StatusCode::is_success(response.status_code()));
        
        zip_extract::extract(Cursor::new(response_body), Path::new(&out_dir), true)
            .expect("Failed to extract SFML ZIP file");
    }
    
    for (name, val) in std::env::vars() {
        println!("Var {} = {}", name, val);
    }
    
    for entry in std::fs::read_dir(&format!("{}\\bin", out_dir)).unwrap() {
        let entry = entry.unwrap();
        std::fs::copy(entry.path(), format!("{}\\{}", exe_directory, entry.file_name().to_string_lossy()))
            .unwrap();
    }    

    println!("cargo:rustc-env=SFML_INCLUDE_DIR={}\\include", out_dir);
    println!("cargo:rustc-env=SFML_INCLUDE_DIR={}\\lib", out_dir);

    let dot_cargo_dir = format!("{}\\..\\..\\.cargo", env!("CARGO_MANIFEST_DIR"));
    println!("{}", dot_cargo_dir);
    if let Err(e) = std::fs::create_dir_all(Path::new(&dot_cargo_dir)) {
        if e.kind() != ErrorKind::AlreadyExists {
            panic!("Error \"{}\": {:?}", dot_cargo_dir, e);
        }
    } else {
        use std::fs::File;
        use std::io::prelude::*;

        let mut file = File::create(&format!("{}\\config.toml", dot_cargo_dir)).unwrap();

        let dir = String::from(out_dir).replace("\\", "/");
        file.write_all(
            
            format!(
                "[env]\r\nSFML_INCLUDE_DIR = \"{}/include\"\r\nSFML_LIBS_DIR = \"{}/lib\"\r\n\r\n",
                dir, dir
            ).as_bytes()
        ).unwrap();
    }
}

#[cfg(not(target_env = "msvc"))]
fn download_precompiled_sfml() {
    // TODO
}

