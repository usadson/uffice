// Copyright (C) 2022 Tristan Gerritsen <tristan@thewoosh.org>
// All Rights Reserved.

pub trait Alphabet {
    fn nth(index: usize) -> char;
}

pub struct Latin;
impl Alphabet for Latin {
    fn nth(index: usize) -> char {
        assert!(index < 26, "Invalid value");
        const LETTERS: &'static [char] = &['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z'];
        LETTERS[index]
    }
}
