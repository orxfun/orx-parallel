//! Recursive tree traversal benchmark with controlled topology skew.
//! Simulates balanced vs highly skewed tree structures to stress irregular
//! scheduling and work distribution under recursive expansion.

use criterion::{Criterion, criterion_group, criterion_main};
use orx_criterion::{Experiment, Factors};
use orx_parallel::*;
use rayon::ThreadPoolBuilder;
use std::hint::black_box;

const CPU_MIX_ROUNDS: usize = 40;
fn cpu_mix(x: u64) -> u64 {
    let mut x = black_box(x ^ 0x9E37_79B9_7F4A_7C15);
    for r in 0..CPU_MIX_ROUNDS {
        let salt = black_box((r as u64 + 1) * 0xA076_1D64_78BD_642F);
        x = black_box(x ^ salt);
        x = black_box(x.rotate_left(9).wrapping_mul(0xD6E8_FD9D_79A1_4E3B));
        x = black_box(x ^ (x >> 27));
    }
    x
}

#[derive(Debug, Clone, Copy)]
enum Topology {
    Balanced,
    Skewed,
}

#[derive(Clone, Copy)]
struct InputVariant {
    depth: usize,
    topology: Topology,
}

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["depth", "topology"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            format!("depth{}", self.depth),
            match self.topology {
                Topology::Balanced => "balanced",
                Topology::Skewed => "skewed",
            }
            .to_string(),
        ]
    }
}

#[derive(Debug)]
enum Method {
    Seq,
    Rayon { nt: usize },
    Orx { nt: usize },
    OrxFixed { nt: usize },
    OrxLin { nt: usize },
    OrxFixedLin { nt: usize },
}

impl Factors for Method {
    fn factor_names() -> Vec<&'static str> {
        vec!["method"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![match self {
            Self::Seq => "seq".to_string(),
            Self::Rayon { nt } => format!("rayon-{nt}"),
            Self::Orx { nt } => format!("orx-{nt}"),
            Self::OrxFixed { nt } => format!("orx-fixed-{nt}"),
            Self::OrxLin { nt } => format!("orx-lin-{nt}"),
            Self::OrxFixedLin { nt } => format!("orx-fixed-lin-{nt}"),
        }]
    }
}

#[derive(Clone, Debug)]
struct TreeNode {
    id: u64,
    value: u32,
    children: Vec<TreeNode>,
}

impl TreeNode {
    fn new(id: u64, value: u32, children: Vec<TreeNode>) -> Self {
        Self {
            id,
            value,
            children,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct TreeAgg {
    count: u64,
    sum_value: u64,
    checksum: u64,
}

fn merge_agg(a: TreeAgg, b: TreeAgg) -> TreeAgg {
    TreeAgg {
        count: a.count + b.count,
        sum_value: a.sum_value.wrapping_add(b.sum_value),
        checksum: a.checksum ^ b.checksum,
    }
}

struct Exp;

fn build_balanced_tree(depth: usize, node_id: &mut u64) -> TreeNode {
    if depth == 0 {
        let id = *node_id;
        *node_id += 1;
        TreeNode::new(id, (id ^ 0x9E37_79B9) as u32, vec![])
    } else {
        let id = *node_id;
        *node_id += 1;
        let value = (id ^ 0xDEAD_BEEF) as u32;
        let children = vec![
            build_balanced_tree(depth - 1, node_id),
            build_balanced_tree(depth - 1, node_id),
            build_balanced_tree(depth - 1, node_id),
        ];
        TreeNode::new(id, value, children)
    }
}

fn build_skewed_tree(depth: usize, node_id: &mut u64) -> TreeNode {
    if depth == 0 {
        let id = *node_id;
        *node_id += 1;
        TreeNode::new(id, (id ^ 0x1337_CAFE) as u32, vec![])
    } else {
        let id = *node_id;
        *node_id += 1;
        let value = (id ^ 0xBEEF_DEAD) as u32;

        let mut children = vec![];
        // Create one heavy branch and multiple light leaves
        children.push(build_skewed_tree(depth - 1, node_id));

        for _ in 0..8 {
            let leaf_id = *node_id;
            *node_id += 1;
            children.push(TreeNode::new(
                leaf_id,
                (leaf_id ^ 0x5555_AAAA) as u32,
                vec![],
            ));
        }

        TreeNode::new(id, value, children)
    }
}

fn process_node(node: &TreeNode) -> TreeAgg {
    let value = cpu_mix(node.value as u64);
    let matches = if value & 0xF == 0x7 { 1 } else { 0 };

    TreeAgg {
        count: 1,
        sum_value: value,
        checksum: node.id ^ value.rotate_left(7) ^ matches,
    }
}

fn seq_traverse(root: &TreeNode) -> TreeAgg {
    fn visit(node: &TreeNode) -> TreeAgg {
        let local = process_node(node);
        let child_agg = node
            .children
            .iter()
            .map(visit)
            .reduce(merge_agg)
            .unwrap_or_default();
        merge_agg(local, child_agg)
    }

    visit(root)
}

fn rayon_traverse(root: &TreeNode, num_threads: usize) -> TreeAgg {
    use rayon::prelude::*;

    let pool = ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .unwrap();

    pool.install(|| {
        fn visit(node: &TreeNode) -> TreeAgg {
            let local = process_node(node);
            let child_agg = node
                .children
                .par_iter()
                .map(visit)
                .reduce(TreeAgg::default, merge_agg);
            merge_agg(local, child_agg)
        }

        visit(root)
    })
}

fn orx_traverse(root: &TreeNode, num_threads: usize) -> TreeAgg {
    [root]
        .into_par_recursive(|node| &node.children)
        .num_threads(num_threads)
        .map(process_node)
        .reduce(merge_agg)
        .unwrap_or_default()
}

fn orx_fixed_traverse(root: &TreeNode, num_threads: usize) -> TreeAgg {
    [root]
        .into_par_recursive(|node| &node.children)
        .runner(Runner::fixed(Pool::once(num_threads)))
        .num_threads(num_threads)
        .map(process_node)
        .reduce(merge_agg)
        .unwrap_or_default()
}

fn orx_lin_traverse(root: &TreeNode, num_threads: usize) -> TreeAgg {
    let linearized: Vec<_> = [root]
        .into_par_recursive(|node| &node.children)
        .num_threads(num_threads)
        .collect();

    linearized
        .into_par()
        .num_threads(num_threads)
        .map(process_node)
        .reduce(merge_agg)
        .unwrap_or_default()
}

fn orx_fixed_lin_traverse(root: &TreeNode, num_threads: usize) -> TreeAgg {
    let linearized: Vec<_> = [root]
        .into_par_recursive(|node| &node.children)
        .runner(Runner::fixed(Pool::once(num_threads)))
        .num_threads(num_threads)
        .collect();

    linearized
        .into_par()
        .runner(Runner::fixed(Pool::once(num_threads)))
        .num_threads(num_threads)
        .map(process_node)
        .reduce(merge_agg)
        .unwrap_or_default()
}

impl Experiment for Exp {
    type InputFactors = InputVariant;

    type AlgFactors = Method;

    type Input = TreeNode;

    type Output = TreeAgg;

    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input {
        let mut node_id = 0u64;
        match input_variant.topology {
            Topology::Balanced => build_balanced_tree(input_variant.depth, &mut node_id),
            Topology::Skewed => build_skewed_tree(input_variant.depth, &mut node_id),
        }
    }

    fn execute(
        &mut self,
        _: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        match alg_variant {
            Method::Seq => seq_traverse(input),
            Method::Rayon { nt } => rayon_traverse(input, *nt),
            Method::Orx { nt } => orx_traverse(input, *nt),
            Method::OrxFixed { nt } => orx_fixed_traverse(input, *nt),
            Method::OrxLin { nt } => orx_lin_traverse(input, *nt),
            Method::OrxFixedLin { nt } => orx_fixed_lin_traverse(input, *nt),
        }
    }

    fn expected_output(&self, _: &Self::InputFactors, input: &Self::Input) -> Option<Self::Output> {
        Some(seq_traverse(input))
    }
}

fn run(c: &mut Criterion) {
    let depths = [10, 12];
    let topologies = [Topology::Skewed, Topology::Balanced];
    let treatments: Vec<_> = depths
        .into_iter()
        .flat_map(|depth| topologies.map(|topology| InputVariant { depth, topology }))
        .collect();

    let par_variants = |nt: usize| {
        [
            Method::Rayon { nt },
            Method::Orx { nt },
            Method::OrxFixed { nt },
            Method::OrxLin { nt },
            Method::OrxFixedLin { nt },
        ]
    };

    let mut variants = vec![Method::Seq];
    variants.extend(par_variants(1));
    variants.extend(par_variants(4));
    variants.extend(par_variants(16));

    Exp.bench(c, "recursive_tree_traversal", &treatments, &variants);
}

criterion_group!(benches, run);
criterion_main!(benches);
