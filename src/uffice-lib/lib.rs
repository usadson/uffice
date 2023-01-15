// Copyright (C) 2022 - 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use std::collections::HashMap;

use winit::event::VirtualKeyCode;

pub mod constants;
pub mod namespaces;
pub mod math;
pub mod profiling;

#[derive(Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum KeyState {
    Released,
    Pressed,
    Held,
}

#[derive(Clone, Debug, Default)]
pub struct Keyboard {
    states: HashMap<VirtualKeyCode, KeyState>,
}

impl Keyboard {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get_state(&self, key: VirtualKeyCode) -> KeyState {
        match self.states.get(&key) {
            Some(state) => state.clone(),
            None => KeyState::Released
        }
    }

    pub fn handle_input_event(&mut self, event: &winit::event::KeyboardInput) {
        use winit::event::ElementState;

        if let Some(virtual_key) = event.virtual_keycode.clone() {
            match event.state {
                ElementState::Pressed => self.states.insert(virtual_key, KeyState::Pressed),
                ElementState::Released => self.states.remove(&virtual_key),
            };
        }
    }

    /// Checks if either of the control keys are down.
    pub fn is_control_key_dow(&self) -> bool {
        self.is_down(VirtualKeyCode::LControl) || self.is_down(VirtualKeyCode::RControl)
    }

    pub fn is_down(&self, key: VirtualKeyCode) -> bool {
        self.get_state(key) != KeyState::Released
    }
}
