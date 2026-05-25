use clap::Parser;
#[cfg(feature = "std")]
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::ThreadPoolBuilder;
use rayon::{Scope, scope};
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

#[cfg(not(feature = "std"))]
fn main() {
    panic!("This example requires std");
}

#[derive(Clone)]
struct DirNode {
    id: usize,
    file_count: usize,
    children: Vec<usize>,
}

impl DirNode {
    fn compute_score(&self, work: usize) -> u64 {
        (0..work)
            .map(|j| {
                let n = core::hint::black_box(((self.id + self.file_count + j) % 35) as u64);
                let mut a = 0u64;
                let mut b = 1u64;
                for _ in 0..n {
                    let c = core::hint::black_box(a + b);
                    a = b;
                    b = c;
                }
                a
            })
            .sum()
    }
}

#[derive(Clone)]
struct FileSystem {
    roots: Vec<usize>,
    nodes: Vec<DirNode>,
}

impl FileSystem {
    fn generate(num_nodes: usize, num_roots: usize, max_children: usize, seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let num_roots = num_roots.min(num_nodes.max(1));
        let max_children = max_children.max(1);

        let mut nodes: Vec<_> = (0..num_nodes)
            .map(|id| DirNode {
                id,
                file_count: rng.random_range(1..20),
                children: Vec::new(),
            })
            .collect();

        let roots: Vec<usize> = (0..num_roots).collect();
        let mut open_parents: Vec<usize> = (0..num_roots).collect();

        for child in num_roots..num_nodes {
            if open_parents.is_empty() {
                open_parents.push(rng.random_range(0..child));
            }

            let parent_slot = rng.random_range(0..open_parents.len());
            let parent = open_parents[parent_slot];

            nodes[parent].children.push(child);
            if nodes[parent].children.len() >= max_children {
                open_parents.swap_remove(parent_slot);
            }

            open_parents.push(child);
        }

        Self { roots, nodes }
    }
}

#[derive(Clone, Copy, Debug)]
enum Method {
    Seq,
    Rayon,
    OrxRayonPool,
    OrxNewPool,
    All,
}

impl core::str::FromStr for Method {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "seq" => Ok(Self::Seq),
            "rayon" => Ok(Self::Rayon),
            "orx-rayon" => Ok(Self::OrxRayonPool),
            "orx-new" => Ok(Self::OrxNewPool),
            "all" => Ok(Self::All),
            _ => Err(format!(
                "unknown method: {s}; expected one of seq|rayon|orx-rayon|orx-new|all"
            )),
        }
    }
}

#[derive(Parser, Debug)]
struct Args {
    /// Number of nodes (directories).
    #[arg(long, default_value_t = 40_000)]
    nodes: usize,
    /// Number of root directories.
    #[arg(long, default_value_t = 50)]
    roots: usize,
    /// Max children per directory.
    #[arg(long, default_value_t = 8)]
    max_children: usize,
    /// Work per visited node.
    #[arg(long, default_value_t = 250)]
    work: usize,
    /// Seed for synthetic file-system generation.
    #[arg(long, default_value_t = 42)]
    seed: u64,
    /// Which method to run: seq | rayon | orx-rayon | orx-new | all.
    #[arg(long, default_value = "all")]
    method: Method,
    /// Number of measured repetitions.
    #[arg(long, default_value_t = 5)]
    repetitions: usize,
    /// Number of warmup runs (not included in stats).
    #[arg(long, default_value_t = 1)]
    warmup: usize,
    /// ORX num threads (0 = auto).
    #[arg(long, default_value_t = 0)]
    num_threads: usize,
    /// ORX chunk size (0 = auto).
    #[arg(long, default_value_t = 0)]
    chunk_size: usize,
    /// Print ORX workload diagnostics.
    #[arg(long, default_value_t = false)]
    diagnostics: bool,
}

fn seq_sum(fs: &FileSystem, work: usize) -> u64 {
    let mut stack = fs.roots.clone();
    let mut sum = 0u64;

    while let Some(idx) = stack.pop() {
        let node = &fs.nodes[idx];
        sum += node.compute_score(work);
        stack.extend(node.children.iter().copied());
    }

    sum
}

fn rayon_sum(fs: &FileSystem, work: usize) -> u64 {
    fn spawn_job<'a>(
        scope: &Scope<'a>,
        fs: &'a FileSystem,
        idx: usize,
        work: usize,
        sum: &'a AtomicU64,
    ) {
        scope.spawn(move |scope| {
            let node = &fs.nodes[idx];
            for child in node.children.iter().copied() {
                spawn_job(scope, fs, child, work, sum);
            }
            sum.fetch_add(node.compute_score(work), Ordering::Relaxed);
        });
    }

    let sum = AtomicU64::new(0);
    scope(|scope| {
        for root in fs.roots.iter().copied() {
            spawn_job(scope, fs, root, work, &sum);
        }
    });

    sum.load(Ordering::Relaxed)
}

#[cfg(feature = "std")]
fn orx_sum(fs: &FileSystem, work: usize, args: &Args) -> u64 {
    let nt = pool_nt(args.num_threads);
    let pool = ThreadPoolBuilder::new()
        .num_threads(nt)
        .build()
        .expect("failed to build rayon thread pool");

    let input = fs
        .roots
        .iter()
        .copied()
        .into_par_recursive(|idx| fs.nodes[*idx].children.iter().copied())
        .pool(&pool)
        .num_threads(args.num_threads)
        .chunk_size(args.chunk_size)
        .map(|idx| fs.nodes[idx].compute_score(work));

    if args.diagnostics {
        input
            .runner_with_diagnostics()
            .reduce(|a, b| a + b)
            .unwrap_or(0)
    } else {
        input.reduce(|a, b| a + b).unwrap_or(0)
    }
}

#[cfg(feature = "std")]
fn pool_nt(num_threads: usize) -> usize {
    if num_threads == 0 {
        std::thread::available_parallelism()
            .map(usize::from)
            .unwrap_or(1)
            .max(1)
    } else {
        num_threads
    }
}

#[cfg(feature = "std")]
fn orx_sum_new_pool(fs: &FileSystem, work: usize, args: &Args) -> u64 {
    let nt = pool_nt(args.num_threads);
    let pool = SimplePool::with_max_num_threads(NonZeroUsize::new(nt).expect(">0"));

    let input = fs
        .roots
        .iter()
        .copied()
        .into_par_recursive(|idx| fs.nodes[*idx].children.iter().copied())
        .pool(pool)
        .num_threads(args.num_threads)
        .chunk_size(args.chunk_size)
        .map(|idx| fs.nodes[idx].compute_score(work));

    if args.diagnostics {
        input
            .runner_with_diagnostics()
            .reduce(|a, b| a + b)
            .unwrap_or(0)
    } else {
        input.reduce(|a, b| a + b).unwrap_or(0)
    }
}

fn run_one(name: &str, reps: usize, mut f: impl FnMut() -> u64) -> (u64, f64, f64, f64) {
    let mut times_ms = Vec::with_capacity(reps);
    let mut last_sum = 0u64;

    for _ in 0..reps {
        let start = Instant::now();
        last_sum = f();
        let elapsed_ms = start.elapsed().as_secs_f64() * 1_000.0;
        times_ms.push(elapsed_ms);
    }

    let avg = times_ms.iter().sum::<f64>() / times_ms.len() as f64;
    let min = times_ms.iter().copied().fold(f64::INFINITY, f64::min);
    let max = times_ms.iter().copied().fold(0.0, f64::max);

    println!(
        "{name:<12} | avg={avg:>8.3} ms | min={min:>8.3} ms | max={max:>8.3} ms | sum={last_sum}"
    );

    (last_sum, avg, min, max)
}

#[cfg(feature = "std")]
fn main() {
    let args = Args::parse();

    println!("\nRecursive tuning workload");
    println!(
        "nodes={} roots={} max_children={} work={} seed={} reps={} warmup={} method={:?} num_threads={} chunk_size={} diagnostics={}",
        args.nodes,
        args.roots,
        args.max_children,
        args.work,
        args.seed,
        args.repetitions,
        args.warmup,
        args.method,
        args.num_threads,
        args.chunk_size,
        args.diagnostics
    );

    let fs = FileSystem::generate(args.nodes, args.roots, args.max_children, args.seed);

    let baseline = seq_sum(&fs, args.work);
    println!("baseline seq sum = {baseline}");

    let selected: &[Method] = match args.method {
        Method::All => &[
            Method::Seq,
            Method::Rayon,
            Method::OrxRayonPool,
            Method::OrxNewPool,
        ],
        Method::Seq => &[Method::Seq],
        Method::Rayon => &[Method::Rayon],
        Method::OrxRayonPool => &[Method::OrxRayonPool],
        Method::OrxNewPool => &[Method::OrxNewPool],
    };

    for method in selected {
        for _ in 0..args.warmup {
            let _ = match method {
                Method::Seq => seq_sum(&fs, args.work),
                Method::Rayon => rayon_sum(&fs, args.work),
                Method::OrxRayonPool => orx_sum(&fs, args.work, &args),
                Method::OrxNewPool => orx_sum_new_pool(&fs, args.work, &args),
                Method::All => unreachable!(),
            };
        }
    }

    println!("\nMeasured runs:");

    let mut reference: Option<u64> = None;
    for method in selected {
        let (sum, _, _, _) = match method {
            Method::Seq => run_one("seq", args.repetitions, || seq_sum(&fs, args.work)),
            Method::Rayon => run_one("rayon", args.repetitions, || rayon_sum(&fs, args.work)),
            Method::OrxRayonPool => run_one("orx-rayon", args.repetitions, || {
                orx_sum(&fs, args.work, &args)
            }),
            Method::OrxNewPool => run_one("orx-new", args.repetitions, || {
                orx_sum_new_pool(&fs, args.work, &args)
            }),
            Method::All => unreachable!(),
        };

        if let Some(expected) = reference {
            assert_eq!(sum, expected, "sum mismatch for method {method:?}");
        } else {
            reference = Some(sum);
        }
    }

    println!("\nAll selected methods produced identical sums.");
}
