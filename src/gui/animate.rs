// Copyright (C) 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use std::{time::Instant, ops::Range};
use uffice_lib::math;
use crate::user_settings::{SettingChangeSubscriber, SettingChangeNotification, SettingName};

use super::Position;

#[derive(Clone, Debug)]
pub enum EasingFunction {
    /// Not an easing function, but will act as if the animation completed
    /// immediately.
    DisabledAnimations,

    #[allow(dead_code)]
    Linear,

    #[allow(dead_code)]
    CubicBezier(Position<f32>, Position<f32>, Position<f32>, Position<f32>),

    EaseOutQuadratic,
}

impl EasingFunction {

    /// Apply the easing function to the given input.
    pub fn apply(&self, x: f32) -> f32 {
        assert!(x >= 0.0);
        assert!(x <= 1.0);

        match self {
            EasingFunction::DisabledAnimations => 1.0,
            EasingFunction::Linear => x,
            EasingFunction::CubicBezier(p0, p1, p2, p3) => {
                let t1 = 1.0 - x;
                let t2 = t1 * t1;
                let t3 = t2 * t1;

                let pt =
                    p0.clone() * t3 +
                    p1.clone() * 3.0 * t2 * x +
                    p2.clone() * 3.0 * t1 * x * x +
                    p3.clone() * 3.0 * x * x * x;

                pt.x
            }
            EasingFunction::EaseOutQuadratic => 1.0 - (1.0 - x) * (1.0 - x),
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_easing() {
        let function = EasingFunction::Linear;

        assert_eq!(function.apply(0.0), 0.0);
        assert_eq!(function.apply(0.1), 0.1);
        assert_eq!(function.apply(0.2), 0.2);
        assert_eq!(function.apply(0.5), 0.5);
        assert_eq!(function.apply(0.75), 0.75);
        assert_eq!(function.apply(1.0), 1.0);
    }

    #[test]
    fn test_ease_out_quadratic() {
        let function = EasingFunction::EaseOutQuadratic;

        assert_eq!(function.apply(0.0), 0.0);
        assert_eq!(function.apply(0.3), 0.51);
        assert_eq!(function.apply(0.5), 0.75);
        assert_eq!(function.apply(0.75), 0.9375);
        assert_eq!(function.apply(1.0), 1.0);
    }
}

#[derive(Debug)]
pub struct Animator {
    pub easing_function: EasingFunction,
    begin: Instant,
    delay_ms: f32,
    finished: bool,
}

/// An object that may contain animations.
pub trait Animated {
    /// Does the object have an animation right now?
    fn has_running_animation(&self) -> bool;
}

impl Animator {
    pub fn new_with_delay(delay_ms: f32, easing_function: EasingFunction) -> Self {
        Self {
            easing_function,
            begin: Instant::now(),
            delay_ms,
            finished: true,
        }
    }

    pub fn reset(&mut self) {
        self.begin = Instant::now();
        self.finished = false;
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }

    pub fn update(&mut self) -> f32 {
        let now = Instant::now();
        let diff = now.duration_since(self.begin);

        if diff.as_millis() > self.delay_ms as u128 {
            self.finished = true;
            return 1.0;
        }

        let value = diff.as_millis() as f32 / self.delay_ms;

        return if value > 1.0 {
            self.finished = true;
            1.0
        } else {
            self.easing_function.apply(value)
        }
    }
}

#[derive(Debug)]
pub struct InterpolatedValue {
    animator: Animator,
    start_value: f32,
    end_value: f32,
    bounds: Range<f32>
}

impl Animated for InterpolatedValue {
    fn has_running_animation(&self) -> bool {
        !self.animator.is_finished()
    }
}

impl InterpolatedValue {
    pub fn new(start_value: f32, duration_ms: f32, easing_function: EasingFunction, bounds: Range<f32>) -> Self {
        Self {
            animator: Animator::new_with_delay(duration_ms, easing_function),
            start_value,
            end_value: start_value,
            bounds
        }
    }

    pub fn change(&mut self, new_value: f32) {
        self.start_value = self.get();
        self.end_value = match new_value {
            d if d < self.bounds.start => self.bounds.start,
            d if d > self.bounds.end => self.bounds.end,
            d => d,
        };
        self.animator.reset();
    }

    // Returns whether or not it has changed.
    pub fn increase(&mut self, delta: f32) -> bool {
        match self.end_value + delta {
            new_value if new_value < self.bounds.start => false,
            new_value if new_value > self.bounds.end => false,
            new_value => {
                self.change(new_value);
                true
            }
        }
    }

    pub fn get(&mut self) -> f32 {
        math::lerp_precise_f32(self.start_value, self.end_value, self.animator.update())
    }

    pub fn set_easing_function(&mut self, function: EasingFunction) {
        self.animator.easing_function = function;
    }
}

/// The zoom levels the user can step through using control + or control -.
const ZOOM_LEVELS: [f32; 31] = [
    0.001, 0.002, 0.003, 0.004, 0.005, 0.0067, 0.075,
    0.1, 0.2, 0.3, 0.4, 0.5, 0.67, 0.8, 0.9,
    1.0,
    1.1, 1.2, 1.33, 1.5, 1.7,
    2.0, 2.5,
    3.0,
    4.0,
    5.0,
    6.7,
    7.5,
    10.0,
    15.0,
    20.0
];

/// Zoom animation speed/duration in milliseconds.
/// TODO: Change this to from f32 to Duration.
const ZOOM_ANIMATION_SPEED: f32 = 150.0;

const DEFAULT_ZOOM_LEVEL_INDEX: usize = 15;

const ZOOM_EASING_FUNCTION: EasingFunction = EasingFunction::EaseOutQuadratic;

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
            zoom_level: InterpolatedValue::new(ZOOM_LEVELS[DEFAULT_ZOOM_LEVEL_INDEX], ZOOM_ANIMATION_SPEED, ZOOM_EASING_FUNCTION, 0.0..f32::MAX),
        }
    }

    /// Steps to the next zoom level, if any.
    /// For example, when the current zoom level is 1.5, it will move to 1.7.
    pub fn increase_zoom_level(&mut self) -> bool {
        let next_zoom_index = self.zoom_index + 1;
        if next_zoom_index >= ZOOM_LEVELS.len() {
            return false;
        }

        self.zoom_index = next_zoom_index;
        self.zoom_level.change(ZOOM_LEVELS[next_zoom_index]);

        return true;
    }

    /// Steps to the previous zoom level, if any.
    /// For example, when the current zoom level is 1.7, it will move to 1.5.
    pub fn decrease_zoom_level(&mut self) -> bool {
        if self.zoom_index == 0 {
            return false;
        }

        let next_zoom_index = self.zoom_index - 1;
        self.zoom_index = next_zoom_index;
        self.zoom_level.change(ZOOM_LEVELS[next_zoom_index]);

        return true;
    }

    /// Gets the zoom factor, determining how zoomed in or out the view should
    /// be.
    pub fn zoom_factor(&mut self) -> f32 {
        self.zoom_level.get()
    }
}

impl Animated for Zoomer {
    fn has_running_animation(&self) -> bool {
        !self.zoom_level.animator.is_finished()
    }
}

impl SettingChangeSubscriber for Zoomer {
    fn settings_loaded(&mut self, settings: &crate::user_settings::UserSettings) {
        self.zoom_level.set_easing_function(
            if settings.setting_enable_animations() {
                ZOOM_EASING_FUNCTION
            } else {
                EasingFunction::DisabledAnimations
            }
        );
    }

    fn setting_changed(&mut self, notification: &SettingChangeNotification) {
        if notification.setting_name != SettingName::EnableAnimations {
            return;
        }

        self.settings_loaded(notification.settings);
    }
}
