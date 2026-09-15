use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::{format, vec};

pub fn inputs_res(n: usize, error_idx: Option<usize>) -> Vec<Result<String, Vec<char>>> {
    let error_idx = error_idx.unwrap_or(usize::MAX);
    (0..n)
        .map(|i| match i == error_idx {
            true => Err(vec!['a']),
            false => Ok(i.to_string()),
        })
        .collect()
}

pub fn inputs(n: usize) -> Vec<String> {
    (0..n).map(|i| i.to_string()).collect()
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
            b: th_idx.to_string(),
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
