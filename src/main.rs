// Copyright (C) 2022 - 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use clap::Parser;

mod application;
mod color_parser;
mod drawing_ml;
mod error;
mod fonts;
mod gui;
mod platform;
mod relationships;
mod style;
mod text_settings;
mod word_processing;
mod unicode;
mod user_settings;
mod wp;

pub const WORD_PROCESSING_XML_NAMESPACE: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";

#[derive(Parser, Debug, Default, Clone)]
pub struct CommandLineArguments {
    /// The files to open.
    files: Vec<String>,
}

fn main() {
    dotenv::dotenv().expect("Failed to load .env");

    println!(">> Uffice <<");

    let mut args = CommandLineArguments::parse();

    //#[cfg(debug_assertions)]
    if args.files.is_empty() {
        if let Ok(test_file) = std::env::var("UFFICE_TEST_FILE") {
            args.files.push(test_file);
        }
    }

    gui::app::run(|window, event_loop_proxy| {
        Box::new(application::App::new(window, event_loop_proxy, args.files))
    });
}
