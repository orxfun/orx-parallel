use crate::load_status::{NodeStatusPar, NodeStatusSeq};
use crate::utils::run;
use crate::{node::Node, node_storage::NodesStorage};
use clap::Parser;
use orx_concurrent_recursive_iter::Queue;
use orx_imp_vec::{ImpVec, PinnedVec};
use orx_parallel::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rayon::{Scope, ThreadPoolBuilder};
use std::sync::atomic::{AtomicU64, Ordering};

mod load_status;
mod node;
mod node_storage;
mod utils;

#[derive(Parser)]
struct Args {
    /// Amount of work (num times Fibonacci will be repeated).
    #[arg(long, default_value_t = 1)]
    amount_of_work: usize,
}

fn seq(storage: &NodesStorage, roots: &[&Node], amount_of_work: usize) -> (u64, usize) {
    let mut status = NodeStatusSeq::new(storage.all_nodes.len(), roots);

    let tasks: ImpVec<_> = roots.iter().copied().collect();
    let mut sum = 0;

    for i in 0.. {
        match tasks.get(i) {
            None => break,
            Some(node) => {
                if status.start_processing(node) {
                    // extend
                    for s in &node.children_symbols {
                        let child = storage.get_relevant_node(s);
                        match status.load_child(child) {
                            false => continue,
                            true => tasks.imp_push(child),
                        }
                    }

                    // process
                    let value = node.compute(amount_of_work);
                    sum += value;
                }
            }
        }
    }

    (sum, status.num_processed())
}

fn rayon(storage: &NodesStorage, roots: &[&Node], amount_of_work: usize) -> (u64, usize) {
    fn spawn_job<'a>(
        scope: &Scope<'a>,
        storage: &'a NodesStorage,
        status: &'a NodeStatusPar,
        node: &'a Node,
        amount_of_work: usize,
        sum: &'a AtomicU64,
    ) {
        scope.spawn(move |scope| {
            if status.start_processing(node) {
                for s in &node.children_symbols {
                    let child = storage.get_relevant_node(s);
                    if status.load_child(child) {
                        spawn_job(scope, storage, status, child, amount_of_work, sum);
                    }
                }

                let value = node.compute(amount_of_work);
                sum.fetch_add(value, Ordering::Relaxed);
            }
        });
    }

    let status = NodeStatusPar::new(storage.all_nodes.len(), roots);
    let sum = AtomicU64::new(0);

    let num_threads = std::thread::available_parallelism()
        .map(|x| x.get())
        .unwrap_or(1);

    let pool = ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .unwrap();

    pool.scope(|scope| {
        for root in roots {
            spawn_job(scope, storage, &status, root, amount_of_work, &sum);
        }
    });

    (sum.load(Ordering::Relaxed), status.num_processed())
}

pub fn orx(
    storage: &NodesStorage,
    roots: &[&Node],
    amount_of_work: usize,
    exact: bool,
) -> (u64, usize) {
    fn get_extend<'x, 'b>(
        storage: &'x NodesStorage,
        status: &'x NodeStatusPar,
    ) -> impl Fn(&&'b Node, &Queue<&'b Node>)
    where
        'x: 'b,
    {
        |node: &&'b Node, queue: &Queue<&'b Node>| {
            if status.start_processing(node) {
                for s in &node.children_symbols {
                    let child = storage.get_relevant_node(s);
                    if status.load_child(child) {
                        queue.push(child);
                    }
                }
            }
        }
    }

    let status = NodeStatusPar::new(storage.all_nodes.len(), roots);
    let extend = get_extend(storage, &status);

    let exact_len = exact.then_some(storage.all_nodes.len());

    let sum = roots
        .iter()
        .copied()
        .into_rec_par(extend, exact_len)
        .map(|x| x.compute(amount_of_work))
        .reduce(|a, b| a + b)
        .unwrap();

    (sum, status.num_processed())
}

fn main() {
    let work = Args::parse().amount_of_work;
    let seed = 42;

    let log = |(sum, count): &(u64, usize)| println!("  count = {count}\n  sum = {sum}");

    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    let storage = NodesStorage::new(10_000, &mut rng);
    let roots = storage.get_roots(20, &mut rng);

    _ = run("seq", || seq(&storage, &roots, work), log);
    _ = run("orx", || orx(&storage, &roots, work, false), log);
    _ = run("rayon", || rayon(&storage, &roots, work), log);
    _ = run("orx_exact", || orx(&storage, &roots, work, true), log);

    // _ = run("try_orx", || try_orx(&storage, &roots, work, true), log);
}

fn try_orx(
    storage: &NodesStorage,
    roots: &[&Node],
    amount_of_work: usize,
    exact: bool,
) -> (u64, usize) {
    fn get_extend<'x, 'b>(
        storage: &'x NodesStorage,
        status: &'x NodeStatusPar,
    ) -> impl Fn(&&'b Node, &Queue<&'b Node>)
    where
        'x: 'b,
    {
        |node: &&'b Node, queue: &Queue<&'b Node>| {
            if status.start_processing(node) {
                for s in &node.children_symbols {
                    let child = storage.get_relevant_node(s);
                    if status.load_child(child) {
                        queue.push(child);
                    }
                }
            }
        }
    }

    let status = NodeStatusPar::new(storage.all_nodes.len(), roots);
    let extend = get_extend(storage, &status);

    let exact_len = exact.then_some(storage.all_nodes.len());
    let sum = roots
        .iter()
        .copied()
        .into_rec_par(extend, exact_len)
        .map(|x| x.compute(amount_of_work))
        .runner_with_diagnostics()
        .reduce(|a, b| a + b)
        .unwrap();

    (sum, status.num_processed())
}
