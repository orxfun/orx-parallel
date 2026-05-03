use crate::node::Node;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct NodeStatusPar {
    loaded: Vec<AtomicBool>,
    processed: Vec<AtomicBool>,
}

impl NodeStatusPar {
    pub fn new(len: usize, roots: &[&Node]) -> Self {
        let loaded = (0..len)
            .map(|idx| roots.iter().any(|x| x.id == idx).into())
            .collect();
        Self {
            loaded,
            processed: (0..len).map(|_| false.into()).collect(),
        }
    }

    pub fn start_processing(&self, node: &Node) -> bool {
        match self.processed[node.id].compare_exchange(
            false,
            true,
            Ordering::AcqRel,
            Ordering::Relaxed,
        ) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn load_child(&self, node: &Node) -> bool {
        match self.processed[node.id].load(Ordering::Relaxed) {
            true => false,
            false => {
                match self.loaded[node.id].compare_exchange(
                    false,
                    true,
                    Ordering::AcqRel,
                    Ordering::Relaxed,
                ) {
                    Ok(_) => true,
                    Err(_) => false,
                }
            }
        }
    }

    pub fn num_processed(&self) -> usize {
        self.processed
            .iter()
            .filter(|x| x.load(Ordering::Relaxed))
            .count()
    }
}

pub struct NodeStatusSeq {
    loaded: Vec<bool>,
    processed: Vec<bool>,
}

impl NodeStatusSeq {
    pub fn new(len: usize, roots: &[&Node]) -> Self {
        let loaded = (0..len)
            .map(|idx| roots.iter().any(|x| x.id == idx))
            .collect();

        Self {
            loaded,
            processed: (0..len).map(|_| false).collect(),
        }
    }

    pub fn start_processing(&mut self, node: &Node) -> bool {
        let processed = self.processed.get_mut(node.id).unwrap();
        match *processed {
            true => false,
            false => {
                *processed = true;
                true
            }
        }
    }

    pub fn load_child(&mut self, child: &Node) -> bool {
        match self.processed[child.id] {
            true => false,
            false => {
                let loaded = self.loaded.get_mut(child.id).unwrap();
                match loaded {
                    true => false,
                    false => {
                        *loaded = true;
                        true
                    }
                }
            }
        }
    }

    pub fn num_processed(&self) -> usize {
        self.processed.iter().filter(|x| **x).count()
    }
}
