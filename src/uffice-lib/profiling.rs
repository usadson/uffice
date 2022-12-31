// Copyright (C) 2022 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

use std::{time::{Instant, Duration}, sync::{Arc, Mutex}};

pub struct ProfileFrame {
    end_time: Arc<Mutex<Instant>>,
}

impl Drop for ProfileFrame {
    fn drop(&mut self) {
        *self.end_time.lock().unwrap() = Instant::now();
    }
}

struct ProfileEntry {
    name: String,
    begin_time: Instant,
    end_time: Arc<Mutex<Instant>>,
}

pub struct Profiler {
    name: String,
    begin_time: Instant,
    entries: Vec<ProfileEntry>
}

fn print_formatted(duration: Duration) {
    let seconds = duration.as_secs();
    if seconds == 0 {
        if duration.as_millis() == 0 {
            println!("{} ns", duration.as_nanos());
        } else {
            println!("{} ms", duration.as_millis());
        }
        return;
    }

    let hours = seconds % 3600;
    let minutes = (seconds % 3600) / 60;
    let seconds = seconds % 60;
    println!("{}:{}:{}.{}", hours, minutes, seconds, duration.as_millis());
}

impl Profiler {
    pub fn new(name: String) -> Self {
        Self {
            name,
            begin_time: Instant::now(),
            entries: Vec::new(),
        }
    }

    pub fn frame(&mut self, name: String) -> ProfileFrame {
        let entry = ProfileEntry {
            name,
            begin_time: Instant::now(),
            end_time: Arc::new(Mutex::new(Instant::now()))
        };
        let end_time = entry.end_time.clone();
        self.entries.push(entry);
        ProfileFrame { end_time }
    }

    pub fn stats(&self) {
        print!("[Profiler] Total Time of {}: ", self.name);
        print_formatted(self.begin_time.elapsed());
        for entry in &self.entries {
            print!("[Profiler]   {}: ", entry.name);
            print_formatted(entry.end_time.lock().unwrap().duration_since(entry.begin_time));
        }
    }
}

impl Drop for Profiler {
    fn drop(&mut self) {
        self.stats();
    }
}

#[macro_export]
macro_rules! profile_expr {
    ($profiler:expr, $name:literal, $expr:expr) => {
        {
            let _frame = $profiler.frame(String::from($name));
            let result = $expr;
            drop(_frame);
            result
        }
    };
}


