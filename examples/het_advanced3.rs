/// Heterogeneous computation case from external repo
/// (https://github.com/orxfun/computation-experiments/tree/parallelization-over-heterogeneous-work)
///
/// The computation:
/// - 1000 work items, each containing a Vec of String
/// - Items 400-440 have 200K strings (heavy), others have 10K strings (light)
/// - Task: clone strings, sort them, get length of first string
/// - This creates a 20× heterogeneity (heavy to light ratio)
///
/// Variants tested:
/// - Sequential
/// - Rayon
/// - Chili (recursive work stealing)
/// - Paralight (range work stealing)
/// - Fixed-1, Fixed-Auto, AdaptiveChunkRunner (orx-parallel)
use clap::{Parser, ValueEnum};
use orx_parallel::*;
use rayon::ThreadPoolBuilder;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::time::Instant;

#[derive(Clone, Copy, Debug, ValueEnum)]
#[value(rename_all = "kebab-case")]
enum Method {
    Seq,
    Rayon,
    Chili,
    OrxFixed1,
    OrxFixed,
    Orx,
    Orx1,
}

#[derive(Parser, Debug)]
struct Args {
    /// Number of threads for parallel methods.
    #[arg(long, default_value_t = 8)]
    num_threads: usize,

    /// Method variant to run.
    #[arg(long, value_enum, default_value_t = Method::Orx)]
    method: Method,

    /// Number of runs to average.
    #[arg(long, default_value_t = 5)]
    runs: usize,

    /// Enables diagnostics printing for AdaptiveChunkRunner.
    #[arg(long, default_value_t = false)]
    diagnostics: bool,
}

#[derive(Clone)]
struct WorkItem {
    strings: Vec<String>,
}

fn build_workload() -> Vec<WorkItem> {
    (1..=1000)
        .map(|i| {
            let size = if (400..=440).contains(&i) {
                200_000
            } else {
                10_000
            };
            WorkItem {
                strings: (0..size).map(|x| x.to_string()).collect(),
            }
        })
        .collect()
}

fn do_work(item: &WorkItem) -> usize {
    let mut x = item.strings.clone();
    x.sort();
    x.first().unwrap().len()
}

fn run_seq(work: &[WorkItem]) -> usize {
    work.iter().map(do_work).max().unwrap_or(0)
}

fn run_rayon(work: &[WorkItem], num_threads: usize) -> usize {
    let pool = ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .expect("failed to build rayon thread pool");

    pool.install(|| work.par_iter().map(do_work).max().unwrap_or(0))
}

fn run_chili(work: &[WorkItem]) -> usize {
    fn process_with_chili<'s, T: Send + Sync + 'static, O: Send + Sync + Ord + 'static>(
        scope: &mut chili::Scope<'s>,
        work: &[T],
        out: &mut [O],
        cb: impl Fn(&T) -> O + Send + Sync + Copy,
    ) {
        let len = work.len();
        if len == 1 {
            out[0] = cb(&work[0]);
            return;
        }
        let mid = len / 2;
        let (a_in, b_in) = work.split_at(mid);
        let (a_out, b_out) = out.split_at_mut(mid);
        scope.join(
            |scope| process_with_chili(scope, a_in, a_out, cb),
            |scope| process_with_chili(scope, b_in, b_out, cb),
        );
    }

    let pool = chili::ThreadPool::new();
    let mut scope = pool.scope();

    let mut out: Vec<usize> = vec![0; work.len()];
    process_with_chili(&mut scope, work, &mut out, do_work);

    out.into_iter().max().unwrap_or(0)
}

fn run_orx(
    work: &[WorkItem],
    num_threads: usize,
    diagnostics: bool,
    chunk_size: usize,
    adaptive: bool,
) -> usize {
    let par = work
        .par()
        .num_threads(num_threads)
        .chunk_size(chunk_size)
        .map(do_work);

    match (adaptive, diagnostics) {
        (false, false) => par.runner(Runner::fixed(Pool::once(num_threads))).max(),
        (false, true) => par
            .runner(Runner::fixed(Pool::once(num_threads)))
            .runner_with_diagnostics()
            .max(),
        (true, false) => par.max(),
        (true, true) => par.runner_with_diagnostics().max(),
    }
    .unwrap_or(0)
}

fn run_selected_method(args: &Args, work: &[WorkItem]) -> usize {
    match args.method {
        Method::Seq => run_seq(work),
        Method::Rayon => run_rayon(work, args.num_threads),
        Method::Chili => run_chili(work),
        Method::OrxFixed => run_orx(work, args.num_threads, args.diagnostics, 0, false),
        Method::OrxFixed1 => run_orx(work, args.num_threads, args.diagnostics, 1, false),
        Method::Orx => run_orx(work, args.num_threads, args.diagnostics, 0, true),
        Method::Orx1 => run_orx(work, args.num_threads, args.diagnostics, 1, true),
    }
}

fn main() {
    let args = Args::parse();
    assert!(args.num_threads > 0, "num_threads must be positive");
    assert!(args.runs > 0, "runs must be positive");

    let work = build_workload();
    let expected = run_seq(&work);

    let mut times_ms = Vec::new();

    for run_idx in 0..args.runs {
        let start = Instant::now();
        let output = run_selected_method(&args, &work);
        let elapsed_ms = start.elapsed().as_secs_f64() * 1_000.0;

        assert_eq!(
            output, expected,
            "output mismatch on run {}: got {}, expected {}",
            run_idx, output, expected
        );

        times_ms.push(elapsed_ms);

        println!(
            "run {} method={:?} threads={} elapsed_ms={:.3}",
            run_idx, args.method, args.num_threads, elapsed_ms
        );
    }

    let avg_ms = times_ms.iter().sum::<f64>() / times_ms.len() as f64;
    let min_ms = times_ms.iter().copied().fold(f64::INFINITY, f64::min);
    let max_ms = times_ms.iter().copied().fold(0.0, f64::max);

    println!();
    println!(
        "summary method={:?} threads={} runs={} avg_ms={:.3} min_ms={:.3} max_ms={:.3}",
        args.method, args.num_threads, args.runs, avg_ms, min_ms, max_ms
    );
}
