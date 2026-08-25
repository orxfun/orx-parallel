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

fn build_tree(depth: usize, fanout: usize, seed: u64) -> Node {
    let children = if depth == 0 {
        vec![]
    } else {
        (0..fanout)
            .map(|index| build_tree(depth - 1, fanout, seed ^ index as u64))
            .collect()
    };
    Node {
        value: seed,
        children,
    }
}

fn work(node: &Node) -> u64 {
    cpu_mix(8, node.value)
}
fn reduce_seq(node: &Node) -> u64 {
    work(node) + node.children.iter().map(reduce_seq).sum::<u64>()
}
fn reduce_rayon(node: &Node) -> u64 {
    work(node) + node.children.par_iter().map(reduce_rayon).sum::<u64>()
}
fn reduce_orx(node: &Node) -> u64 {
    [node]
        .into_par_recursive(|node| &node.children)
        .map(work)
        .sum()
}

impl Experiment for Exp {
    type InputFactors = InputVariant;
    type AlgFactors = Method;
    type Input = Node;
    type Output = u64;

    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input {
        build_tree(input_variant.depth, input_variant.fanout, 42)
    }
    fn execute(
        &mut self,
        _: &Self::InputFactors,
        method: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        match method {
            Method::Seq => reduce_seq(input),
            Method::Rayon => reduce_rayon(input),
            Method::OrxOnce | Method::OrxBasic | Method::OrxRayon => reduce_orx(input),
        }
    }
    fn expected_output(&self, _: &Self::InputFactors, input: &Self::Input) -> Option<Self::Output> {
        Some(reduce_seq(input))
    }
}
