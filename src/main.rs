// Copyright (C) 2022 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

mod application;
mod color_parser;
mod error;
mod fonts;
mod relationships;
mod structured_document_tag;
mod style;
mod text_settings;
mod word_processing;
mod wp;

use sfml::graphics::*;

use structopt::StructOpt;
use style::StyleManager;

use crate::application::Application;

pub const WORD_PROCESSING_XML_NAMESPACE: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";

#[derive(StructOpt, Debug)]
#[structopt(name = "Uffice")]
struct Options {

}

fn main() {
    dotenv::dotenv().expect("Failed to load .env");

    println!(">> Uffice <<");

    let mut app = Application::new(
            std::env::var("UFFICE_TEST_FILE").expect("No file given")
    );
    app.run();
}
