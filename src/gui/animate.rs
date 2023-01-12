// Copyright (C) 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use std::time::Instant;
use uffice_lib::math;
use super::scroll::Scroller;

#[derive(Debug)]
pub struct Animator {
    begin: Instant,
    delay_ms: f32,
}

impl Animator {
    pub fn new() -> Self {
        Self {
            begin: Instant::now(),
            delay_ms: 220.0,
        }
    }

    pub fn new_with_delay(delay_ms: f32) -> Self {
        Self {
            begin: Instant::now(),
            delay_ms,
        }
    }

    pub fn reset(&mut self) {
        self.begin = Instant::now();
    }

    pub fn update(&mut self) -> f32 {
        let now = Instant::now();
        let diff = now.duration_since(self.begin);

        if diff.as_millis() > self.delay_ms as u128 {
            return 1.0;
        }

        let value = diff.as_millis() as f32 / self.delay_ms;

        return if value > 1.0 {
            1.0
        } else {
            value
        }
    }
}

#[derive(Debug)]
pub struct InterpolatedValue {
    animator: Animator,
    start_value: f32,
    end_value: f32,
}

impl InterpolatedValue {
    pub fn new(start_value: f32, duration_ms: f32) -> Self {
        Self {
            animator: Animator::new_with_delay(duration_ms),
            start_value,
            end_value: start_value
        }
    }

    pub fn change(&mut self, new_value: f32) {
        self.start_value = self.get();
        self.end_value = new_value;
        self.animator.reset();
    }

    pub fn get(&mut self) -> f32 {
        Scroller::bound_position(math::lerp_precise_f32(self.start_value, self.end_value, self.animator.update()))
    }
}

/// The zoom levels the user can step through using control + or control -.
const ZOOM_LEVELS: [f32; 19] = [0.1, 0.2, 0.3, 0.4, 0.5, 0.67, 0.8, 0.9, 1.0, 1.1, 1.2, 1.33, 1.5, 1.7, 2.0, 2.5, 3.0, 4.0, 5.0];

/// Zoom animation speed/duration in milliseconds.
/// TODO: Change this to from f32 to Duration.
const ZOOM_ANIMATION_SPEED: f32 = 150.0;

const DEFAULT_ZOOM_LEVEL_INDEX: usize = 4;

#[derive(Debug)]
/// Controls the zoom behavior by processesing Control +/- and
/// Control + Mouse Wheel.
///
/// I cannot think of a beter name than this or "ZoomManager", I'm sorry ;)
pub struct Zoomer {
    zoom_index: usize,
    zoom_level: InterpolatedValue,
}

impl Zoomer {
    pub fn new() -> Self {
        Self {
            zoom_index: DEFAULT_ZOOM_LEVEL_INDEX,
            zoom_level: InterpolatedValue::new(ZOOM_LEVELS[DEFAULT_ZOOM_LEVEL_INDEX], ZOOM_ANIMATION_SPEED),
        }
    }

    /// Steps to the next zoom level, if any.
    /// For example, when the current zoom level is 1.5, it will move to 1.7.
    pub fn increase_zoom_level(&mut self) {
        let next_zoom_index = self.zoom_index + 1;
        if next_zoom_index < ZOOM_LEVELS.len() {
            self.zoom_index = next_zoom_index;
            self.zoom_level.change(ZOOM_LEVELS[next_zoom_index]);
        }
    }

    /// Steps to the previous zoom level, if any.
    /// For example, when the current zoom level is 1.7, it will move to 1.5.
    pub fn decrease_zoom_level(&mut self) {
        if self.zoom_index != 0 {
            let next_zoom_index = self.zoom_index - 1;
            self.zoom_index = next_zoom_index;
            self.zoom_level.change(ZOOM_LEVELS[next_zoom_index]);
        }
    }

    /// Gets the zoom factor, determining how zoomed in or out the view should
    /// be.
    pub fn zoom_factor(&mut self) -> f32 {
        self.zoom_level.get()
    }
}
