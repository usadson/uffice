// Copyright (C) 2022 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use sfml::window::CursorType;

use crate::text_settings::{Rect, Position};

pub trait Interactable {
    fn interation_state(&self) -> &SharedInteractionState;
    fn interation_state_mut(&mut self) -> &mut SharedInteractionState;
    fn on_click(&self, position: Position);
}

pub struct SharedInteractionState {
    pub rects: Vec<Rect>,
    pub cursor_on_hover: Option<CursorType>,

    pub is_hovering: bool,
}

pub struct Link {
    state: SharedInteractionState,
    href: String,
}

impl Link {
    pub fn new(state: SharedInteractionState, href: String) -> Self {
        Self {
            state,
            href
        }
    }

    #[cfg(target_os = "windows")]
    pub fn open_browser(&self, url: &url::Url) {
        use std::process::Command;
        _ = Command::new("cmd.exe")
                .arg("/C")
                .arg("start")
                .arg("")
                .arg(&url.to_string())
                .spawn();
    }

    #[cfg(target_os = "macos")]
    pub fn open_browser(&self, url: &url::Url) {
    }

    #[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "openbsd", 
              target_os = "dragonfly", target_os = "netbsd"))]
    pub fn open_browser(&self, url: &url::Url) {

    }
}

impl Interactable for Link {

    fn interation_state(&self) -> &SharedInteractionState {
        &self.state
    }

    fn interation_state_mut(&mut self) -> &mut SharedInteractionState {
        &mut self.state
    }

    fn on_click(&self, _: Position) {
        match url::Url::parse(&self.href) {
            Err(e) => println!("[Interactable] (Link): \"{}\": {:?}", self.href, e),
            Ok(url) => self.open_browser(&url)
        }
    }

}

