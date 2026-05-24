use criterion::{Criterion, criterion_group, criterion_main};
use enum_iterator::Sequence;
use orx_criterion::{Experiment, Factors};
use orx_parallel::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::{Scope, ThreadPool, ThreadPoolBuilder, scope};
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU64, Ordering};

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

        // Build a forest: first `num_roots` nodes are roots, each next node is
        // attached to an existing parent that still has room for children.
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

fn rayon_sum(pool: &ThreadPool, fs: &FileSystem, work: usize) -> u64 {
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
    pool.install(|| {
        scope(|scope| {
            for root in fs.roots.iter().copied() {
                spawn_job(scope, fs, root, work, &sum);
            }
        });
    });

    sum.load(Ordering::Relaxed)
}

fn orx_sum(pool: &ThreadPool, fs: &FileSystem, work: usize, chunk_size: usize) -> u64 {
    fs.roots
        .iter()
        .copied()
        .into_par_recursive(|idx| fs.nodes[*idx].children.iter().copied())
        .pool(pool)
        .chunk_size(chunk_size)
        .map(|idx| fs.nodes[idx].compute_score(work))
        .reduce(|a, b| a + b)
        .unwrap_or(0)
}

    fn orx_sum_new_pool(pool: &NewPool, fs: &FileSystem, work: usize, chunk_size: usize) -> u64 {
        fs.roots
        .iter()
        .copied()
        .into_par_recursive(|idx| fs.nodes[*idx].children.iter().copied())
        .pool(pool)
        .chunk_size(chunk_size)
        .map(|idx| fs.nodes[idx].compute_score(work))
        .reduce(|a, b| a + b)
        .unwrap_or(0)
    }

#[derive(Clone)]
struct Input {
    num_nodes: usize,
    num_roots: usize,
    max_children: usize,
    num_threads: usize,
    chunk_size: usize,
    work: usize,
}

impl Factors for Input {
    fn factor_names() -> Vec<&'static str> {
        vec!["n", "r", "ch", "nt", "cs", "w"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            self.num_nodes.to_string(),
            self.num_roots.to_string(),
            self.max_children.to_string(),
            self.num_threads.to_string(),
            self.chunk_size.to_string(),
            self.work.to_string(),
        ]
    }
}

#[derive(Debug, Sequence)]
enum Method {
    Seq,
    Rayon,
    OrxRayon,
    OrxNew,
}

impl Factors for Method {
    fn factor_names() -> Vec<&'static str> {
        vec!["method"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            match self {
                Self::Seq => "seq",
                Self::Rayon => "rayon",
                Self::OrxRayon => "orx-rayon",
                Self::OrxNew => "orx-new",
            }
            .to_string(),
        ]
    }
}

struct Exp;

struct BenchInput {
    fs: FileSystem,
    rayon_pool: ThreadPool,
    new_pool: NewPool,
}

impl Experiment for Exp {
    type InputFactors = Input;

    type AlgFactors = Method;

    type Input = BenchInput;

    type Output = u64;

    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input {
        let fs = FileSystem::generate(
            input_variant.num_nodes,
            input_variant.num_roots,
            input_variant.max_children,
            42,
        );

        let rayon_pool = ThreadPoolBuilder::new()
            .num_threads(input_variant.num_threads)
            .build()
            .expect("failed to build rayon thread pool");

        let new_pool = NewPool::with_max_num_threads(
            NonZeroUsize::new(input_variant.num_threads).expect(">0"),
        );

        BenchInput {
            fs,
            rayon_pool,
            new_pool,
        }
    }

    fn execute(
        &mut self,
        input_variant: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        match alg_variant {
            Method::Seq => seq_sum(&input.fs, input_variant.work),
            Method::Rayon => rayon_sum(&input.rayon_pool, &input.fs, input_variant.work),
            Method::OrxRayon => orx_sum(
                &input.rayon_pool,
                &input.fs,
                input_variant.work,
                input_variant.chunk_size,
            ),
            Method::OrxNew => orx_sum_new_pool(
                &input.new_pool,
                &input.fs,
                input_variant.work,
                input_variant.chunk_size,
            ),
        }
    }

    fn validate_output(
        &self,
        input_variant: &Self::InputFactors,
        input: &Self::Input,
        output: &Self::Output,
    ) {
        let expected = seq_sum(&input.fs, input_variant.work);
        assert_eq!(output, &expected);
    }
}

fn run(c: &mut Criterion) {
    let num_threads = [8, 16, 32];
    let chunk_sizes = [0, 256, 1024];

    let treatments: Vec<_> = chunk_sizes
        .into_iter()
        .flat_map(|chunk_size| {
            num_threads.into_iter().flat_map(move |num_threads| {
                [
                    Input {
                        num_nodes: 10_000,
                        num_roots: 20,
                        max_children: 6,
                        num_threads,
                        chunk_size,
                        work: 250,
                    },
                    Input {
                        num_nodes: 40_000,
                        num_roots: 50,
                        max_children: 8,
                        num_threads,
                        chunk_size,
                        work: 250,
                    },
                ]
            })
        })
        .collect();

    let variants = vec![Method::Rayon, Method::OrxRayon, Method::OrxNew];

    Exp.bench(c, "file_system", &treatments, &variants);
}

criterion_group!(benches, run);
criterion_main!(benches);
