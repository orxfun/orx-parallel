use clap::Parser;
use orx_parallel::*;

#[derive(Parser)]
struct Args {
    /// Length of the input array
    #[arg(long, default_value_t = 1_000_000)]
    n: usize,
    /// Magnitude of work per input element
    #[arg(long, default_value_t = 10)]
    work: usize,
    /// Number of threads
    #[arg(long, default_value_t = 0)]
    num_threads: usize,
    /// Chunk size
    #[arg(long, default_value_t = 0)]
    chunk_size: usize,
}

/// Fibonacci as example computation.
fn compute(amount_of_work: usize, n: u64) -> u64 {
    (0..amount_of_work)
        .map(|j| {
            let n = core::hint::black_box((n + j as u64) % 40);
            let mut a = 0;
            let mut b = 1;
            for _ in 0..n {
                let c = a + b;
                a = b;
                b = c;
            }
            a
        })
        .sum()
}

fn main() {
    let args = Args::parse();

    let sum = (0..args.n)
        .par()
        .num_threads(args.num_threads)
        .chunk_size(args.chunk_size)
        .runner_with_diagnostics()
        .map(|i| 2 * i)
        .flat_map(|i| match i.is_multiple_of(2) {
            true => [i, i + 1, i + 2, i + 3, i + 4],
            false => [i, i + 1, usize::MAX, usize::MAX, usize::MAX],
        })
        .filter_map(|i| (i < usize::MAX).then_some(i))
        .map(|i| compute(args.work, i as u64))
        .reduce(|a, b| a + b)
        .unwrap();
    println!("\n\nSUM = {sum}");
}
