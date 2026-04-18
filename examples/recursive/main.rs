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

#[derive(Parser, Debug)]
struct Args {
    /// Amount of work (num times Fibonacci will be repeated).
    #[arg(long, default_value_t = 1_000_000)]
    work: usize,
    /// (only for orx) Number of threads (0 = auto)
    #[arg(long, default_value_t = 0)]
    num_threads: usize,
    /// (only for orx) Chunk size (0 = auto)
    #[arg(long, default_value_t = 0)]
    chunk_size: usize,
    /// (only for orx, try)
    /// When true, first extends & discovers all tasks and then executes.
    /// When false, extension and execution happen together on the fly.
    #[arg(long, default_value_t = false)]
    extended: bool,
    /// When true, only orx variant will execute with defined settings reporting diagnostics.
    /// When false, all variants will be executed and reported.
    #[arg(long, default_value_t = false)]
    diagnostics: bool,
}

fn seq(storage: &NodesStorage, roots: &[&Node], args: &Args) -> (u64, usize) {
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
                    let value = node.compute(args.work);
                    sum += value;
                }
            }
        }
    }

    (sum, status.num_processed())
}

fn rayon(storage: &NodesStorage, roots: &[&Node], args: &Args) -> (u64, usize) {
    fn spawn_job<'a>(
        scope: &Scope<'a>,
        storage: &'a NodesStorage,
        status: &'a NodeStatusPar,
        node: &'a Node,
        args: &'a Args,
        sum: &'a AtomicU64,
    ) {
        scope.spawn(move |scope| {
            if status.start_processing(node) {
                for s in &node.children_symbols {
                    let child = storage.get_relevant_node(s);
                    if status.load_child(child) {
                        spawn_job(scope, storage, status, child, args, sum);
                    }
                }

                let value = node.compute(args.work);
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
            spawn_job(scope, storage, &status, root, args, &sum);
        }
    });

    (sum.load(Ordering::Relaxed), status.num_processed())
}

fn orx_rec(storage: &NodesStorage, roots: &[&Node], args: &Args, exact: bool) -> (u64, usize) {
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
        .into_par_recursive(extend, exact_len)
        .map(|x| x.compute(args.work))
        .reduce(|a, b| a + b)
        .unwrap();

    (sum, status.num_processed())
}

fn orx_extended(storage: &NodesStorage, roots: &[&Node], args: &Args, exact: bool) -> (u64, usize) {
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
        .extend_into_par(extend, exact_len)
        .map(|x| x.compute(args.work))
        .reduce(|a, b| a + b)
        .unwrap();

    (sum, status.num_processed())
}

fn orx_diagnostics(
    storage: &NodesStorage,
    roots: &[&Node],
    args: &Args,
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
    let input = roots.iter().copied();
    let sum = match args.extended {
        true => input
            .extend_into_par(extend, exact_len)
            .num_threads(args.num_threads)
            .chunk_size(args.chunk_size)
            .map(|x| x.compute(args.work))
            .runner_with_diagnostics()
            .reduce(|a, b| a + b)
            .unwrap(),
        false => input
            .into_par_recursive(extend, exact_len)
            .num_threads(args.num_threads)
            .chunk_size(args.chunk_size)
            .map(|x| x.compute(args.work))
            .runner_with_diagnostics()
            .reduce(|a, b| a + b)
            .unwrap(),
    };

    (sum, status.num_processed())
}

fn main() {
    let seed = 42;
    let args = Args::parse();
    println!("\n{args:?}\n\n");

    let log = |(sum, count): &(u64, usize)| println!("  count = {count}\n  sum = {sum}");

    let mut rng = ChaCha8Rng::seed_from_u64(seed);

    let storage = NodesStorage::new(10_000, &mut rng);
    let roots = storage.get_roots(20, &mut rng);

    match args.diagnostics {
        true => {
            _ = run(
                "orx_diagnostics",
                || orx_diagnostics(&storage, &roots, &args, true),
                log,
            );
        }
        false => {
            _ = run("seq", || seq(&storage, &roots, &args), log);
            _ = run("orx_rec", || orx_rec(&storage, &roots, &args, false), log);
            _ = run(
                "orx_rec_exact",
                || orx_rec(&storage, &roots, &args, true),
                log,
            );
            _ = run(
                "orx_extended",
                || orx_extended(&storage, &roots, &args, false),
                log,
            );
            _ = run("rayon", || rayon(&storage, &roots, &args), log);
            _ = run(
                "orx_extended_exact",
                || orx_extended(&storage, &roots, &args, true),
                log,
            );
        }
    }
}
