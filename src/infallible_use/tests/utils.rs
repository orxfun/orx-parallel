use alloc::format;
use alloc::vec::Vec;
use std::string::{String, ToString};

pub fn inputs(n: usize) -> Vec<String> {
    (0..n).map(|x| x.to_string()).collect()
}

#[derive(Clone, Debug)]
pub struct UseValue {
    pub a: usize,
    pub b: String,
}

impl UseValue {
    pub fn new(th_idx: usize) -> Self {
        Self {
            a: 0,
            b: "42".to_string(),
        }
    }

    pub fn mutate(&mut self) {
        self.a += 1;
        match self.a % 3 {
            0 => self.b = format!("{}!", self.b),
            1 => self.b = format!("{}?", self.b),
            _ => self.b = self.b.replace("!", "").replace("?", ""),
        }
    }
}
