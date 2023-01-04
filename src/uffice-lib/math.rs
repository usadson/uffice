// Copyright (C) 2023 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

pub fn lerp_precise_f32(x0: f32, x1: f32, t: f32) -> f32 {
    (1.0 - t) * x0 + t * x1
}
