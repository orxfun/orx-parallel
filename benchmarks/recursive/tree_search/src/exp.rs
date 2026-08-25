use crate::{alg::Method, input::InputVariant};
use bench_helper::runner::cpu_mix;
use orx_criterion::Experiment;
use orx_parallel::*;
use rayon::prelude::*;

pub struct Exp;

pub struct Node {
    value: u64,
    children: Vec<Node>,
}

fn build_tree(depth: usize, fan_out: usize, seed: u64) -> Node {
    let children = if depth == 0 {
        vec![]
    } else {
        (0..fan_out)
            .map(|index| build_tree(depth - 1, fan_out, seed ^ index as u64))
            .collect()
    };
    Node {
        value: cpu_mix(2, seed),
        children,
    }
}

fn matches(node: &Node, threshold: u64) -> bool {
    let node_value = cpu_mix(10, node.value);
    node_value % 10_000 < threshold
}
fn search_seq(node: &Node, threshold: u64) -> usize {
    usize::from(matches(node, threshold))
        + node
            .children
            .iter()
            .map(|child| search_seq(child, threshold))
            .sum::<usize>()
}
fn search_rayon(node: &Node, threshold: u64) -> usize {
    usize::from(matches(node, threshold))
        + node
            .children
            .par_iter()
            .map(|child| search_rayon(child, threshold))
            .sum::<usize>()
}
fn search_orx(node: &Node, threshold: u64) -> usize {
    [node]
        .into_par_recursive(|node| &node.children)
        .filter(|node| matches(node, threshold))
        // .chunk_size(128)
        // .runner_with_diagnostics()
        .count()
}

impl Experiment for Exp {
    type InputFactors = InputVariant;
    type AlgFactors = Method;
    type Input = Node;
    type Output = usize;

    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input {
        build_tree(input_variant.depth, input_variant.fan_out, 42)
    }
    fn execute(
        &mut self,
        input_variant: &Self::InputFactors,
        method: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        match method {
            Method::Seq => search_seq(input, input_variant.threshold),
            Method::Rayon => search_rayon(input, input_variant.threshold),
            Method::OrxOnce | Method::OrxBasic | Method::OrxRayon => {
                search_orx(input, input_variant.threshold)
            }
        }
    }
    fn expected_output(
        &self,
        input_variant: &Self::InputFactors,
        input: &Self::Input,
    ) -> Option<Self::Output> {
        Some(search_seq(input, input_variant.threshold))
    }
}
