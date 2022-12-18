// Copyright (C) 2022 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use notify::Watcher;

fn main() {
    let mut watcher = notify::recommended_watcher(move |res| {
        match res {
            Ok(event) => {
                println!("[Watcher] Event: {:?}", event);
            }
            Err(e) => println!("[Watcher] Failed to watch: {:?}", e),
        }
    }).expect("Failed to instantiate file watcher");

    let path = std::path::Path::new("C:\\");
    
    println!("{:?} => {:?}", path, path.canonicalize());

    if let Ok(md) = path.metadata() {
        println!("Meta: {:?}", md);
    }

    match watcher.watch(path, notify::RecursiveMode::Recursive) {
        Err(err) => {
            println!("{:?}", err);
            return;
        }
        _ => loop {}
    }
}
