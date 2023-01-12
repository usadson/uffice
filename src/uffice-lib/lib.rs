// Copyright (C) 2022 - 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use std::collections::HashMap;

pub mod constants;
pub mod namespaces;
pub mod math;
pub mod profiling;

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum KeyState {
    RELEASED,
    PRESSED,
    HELD,
}

#[derive(Clone, Debug, Default)]
pub struct Keyboard {
    states: HashMap<sfml::window::Key, KeyState>,
}

impl Keyboard {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get_state(&self, key: sfml::window::Key) -> KeyState {
        match self.states.get(&key) {
            Some(state) => state.clone(),
            None => KeyState::RELEASED
        }
    }

    pub fn handle_sfml_event(&mut self, event: &sfml::window::Event) {
        use sfml::window::Event;

        match event {
            Event::KeyPressed { code, alt: _, ctrl: _, shift: _, system: _ } => {
                self.states.insert(code.clone(), KeyState::PRESSED);
            }

            Event::KeyReleased { code, alt: _, ctrl: _, shift: _, system: _ } => {
                self.states.remove(code);
            }

            _ => ()
        }
    }

    /// Checks if either of the control keys are down.
    pub fn is_control_key_dow(&self) -> bool {
        use sfml::window::Key;
        self.is_down(Key::LControl) || self.is_down(Key::RControl)
    }

    pub fn is_down(&self, key: sfml::window::Key) -> bool {
        self.get_state(key) != KeyState::RELEASED
    }
}
