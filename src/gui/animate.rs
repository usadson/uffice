// Copyright (C) 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use std::time::Instant;
use uffice_lib::math;
use super::scroll::Scroller;

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
