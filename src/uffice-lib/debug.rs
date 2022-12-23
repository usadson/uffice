// Copyright (C) 2022 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

pub struct TreeDebugLogger {
    indent: usize
}

macro_rules! tree_debug_log {
    ($($args:expr),*) => {{
        $(
            println!("{}├─ {}", logger, $args);
        )*
    }}
}

impl TreeDebugLogger {

    pub fn prefix(&self) -> &str {
        &self.cached
    }

}

impl Clone for TreeDebugLogger {
    fn clone(&self) -> TreeDebugLogger {
        Self { 
            cached: "│  ".repeat(self.indent + 1),
            indent: self.indent + 1
        }
    }
}

