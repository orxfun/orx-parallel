use crate::node::Node;
use rand::Rng;
use std::{collections::HashSet, marker::PhantomData, ops::Range};

pub struct Tree<T>(PhantomData<T>);

impl<T> Tree<T> {
    pub fn new(
        num_nodes: usize,
        degree: Range<usize>,
        data: fn(usize) -> T,
        rng: &mut impl Rng,
    ) -> Node<T> {
        assert!(num_nodes >= 2);

        let mut leaves = vec![0];
        let mut remaining: Vec<_> = (1..num_nodes).collect();
        let mut edges = vec![];
        let mut out_edges = vec![vec![]; num_nodes];

        while !remaining.is_empty() {
            let leaf_idx = rng.random_range(0..leaves.len());
            let leaf = leaves.remove(leaf_idx);

            let degree = rng.random_range(degree.clone());
            match degree == 0 {
                true => leaves.push(leaf),
                false => {
                    let children_indices: HashSet<_> = (0..degree)
                        .map(|_| rng.random_range(0..remaining.len()))
                        .collect();

                    let mut sorted: Vec<_> = children_indices.iter().copied().collect();
                    sorted.sort();

                    edges.extend(children_indices.iter().map(|c| (leaf, remaining[*c])));
                    out_edges[leaf] = children_indices.iter().map(|c| remaining[*c]).collect();
                    leaves.extend(children_indices.iter().map(|c| remaining[*c]));

                    for idx in sorted.into_iter().rev() {
                        remaining.remove(idx);
                    }
                }
            }
        }

        create_node(&out_edges, 0, data)
    }
}

fn create_node<T>(out_edges: &[Vec<usize>], idx: usize, data: fn(usize) -> T) -> Node<T> {
    let children: Vec<_> = out_edges[idx]
        .iter()
        .map(|child_idx| create_node(out_edges, *child_idx, data))
        .collect();
    let data = data(idx);
    Node {
        idx,
        data,
        children,
    }
}
