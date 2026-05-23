use clap::Parser;
#[cfg(feature = "std")]
use orx_parallel::*;
use std::hint::black_box;

#[cfg(not(feature = "std"))]
fn main() {
    panic!("This example requires std");
}

#[derive(Parser)]
struct Args {
    /// Number of loan applications to score
    #[arg(long, default_value_t = 1_000_000)]
    num_applications: usize,
    /// Number of threads (0 = auto)
    #[arg(long, default_value_t = 0)]
    num_threads: usize,
    /// Chunk size (0 = auto)
    #[arg(long, default_value_t = 0)]
    chunk_size: usize,
}

/// Computes a risk score for a single loan application.
///
/// The credit history length (derived from the applicant id) determines how
/// much work is done per item, producing a naturally uneven workload — exactly
/// the kind of load where diagnostics are most informative.
#[cfg(feature = "std")]
fn compute_risk_score(applicant_id: usize, credit_history_len: usize) -> u64 {
    (0..credit_history_len)
        .map(|month| {
            let n = black_box((applicant_id + month) as u64 % 40);
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

#[cfg(feature = "std")]
fn main() {
    let args = Args::parse();

    // `runner_with_diagnostics()` wraps the runner so that once the parallel
    // computation finishes it prints a summary table and a visual timeline:
    //
    //   - which threads were spawned
    //   - how many task chunks each thread pulled from the input
    //   - a bar chart showing each thread's active time relative to the longest
    //
    // Re-run with different --num-threads and --chunk-size values to see how
    // those settings affect load distribution across threads.
    let total_risk: u64 = (0..args.num_applications)
        .par()
        .num_threads(args.num_threads)
        .chunk_size(args.chunk_size)
        .runner_with_diagnostics()
        .map(|id| compute_risk_score(id, (id % 40) + 1))
        .reduce(|a, b| a + b)
        .unwrap_or(0);

    println!("\nTotal risk score across all applications: {total_risk}");
}
