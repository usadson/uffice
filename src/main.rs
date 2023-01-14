// Copyright (C) 2022 - 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

mod application;
mod color_parser;
mod drawing_ml;
mod error;
mod fonts;
mod gui;
mod relationships;
mod style;
mod text_settings;
mod word_processing;
mod unicode;
mod wp;

pub const WORD_PROCESSING_XML_NAMESPACE: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";

fn main() {
    dotenv::dotenv().expect("Failed to load .env");

    println!(">> Uffice <<");

    gui::app::run(|window, event_loop_proxy| {
        Box::new(application::App::new(window, event_loop_proxy, std::env::var("UFFICE_TEST_FILE").expect("No file given")))
    });
}
