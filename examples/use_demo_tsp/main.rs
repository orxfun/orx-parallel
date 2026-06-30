mod locations;
mod par_with_use;
mod par_without_use;
mod rand_utils;

use crate::{
    locations::locations, par_with_use::run_search_parallel_use_mut,
    par_without_use::run_search_parallel_immutable,
};
use std::{
    alloc::{GlobalAlloc, Layout, System},
    hint::black_box,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

struct TrackingAllocator;

#[global_allocator]
static GLOBAL_ALLOCATOR: TrackingAllocator = TrackingAllocator;

static ALLOC_CALLS: AtomicU64 = AtomicU64::new(0);
static DEALLOC_CALLS: AtomicU64 = AtomicU64::new(0);
static REALLOC_CALLS: AtomicU64 = AtomicU64::new(0);
static ALLOC_BYTES: AtomicU64 = AtomicU64::new(0);
static DEALLOC_BYTES: AtomicU64 = AtomicU64::new(0);
static REALLOC_OLD_BYTES: AtomicU64 = AtomicU64::new(0);
static REALLOC_NEW_BYTES: AtomicU64 = AtomicU64::new(0);

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOC_CALLS.fetch_add(1, Ordering::Relaxed);
        ALLOC_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        DEALLOC_CALLS.fetch_add(1, Ordering::Relaxed);
        DEALLOC_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        REALLOC_CALLS.fetch_add(1, Ordering::Relaxed);
        REALLOC_OLD_BYTES.fetch_add(layout.size() as u64, Ordering::Relaxed);
        REALLOC_NEW_BYTES.fetch_add(new_size as u64, Ordering::Relaxed);
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[derive(Clone, Copy, Default)]
struct AllocationStats {
    alloc_calls: u64,
    dealloc_calls: u64,
    realloc_calls: u64,
    alloc_bytes: u64,
    dealloc_bytes: u64,
    realloc_old_bytes: u64,
    realloc_new_bytes: u64,
}

impl AllocationStats {
    fn reset() {
        ALLOC_CALLS.store(0, Ordering::Relaxed);
        DEALLOC_CALLS.store(0, Ordering::Relaxed);
        REALLOC_CALLS.store(0, Ordering::Relaxed);
        ALLOC_BYTES.store(0, Ordering::Relaxed);
        DEALLOC_BYTES.store(0, Ordering::Relaxed);
        REALLOC_OLD_BYTES.store(0, Ordering::Relaxed);
        REALLOC_NEW_BYTES.store(0, Ordering::Relaxed);
    }

    fn read() -> Self {
        Self {
            alloc_calls: ALLOC_CALLS.load(Ordering::Relaxed),
            dealloc_calls: DEALLOC_CALLS.load(Ordering::Relaxed),
            realloc_calls: REALLOC_CALLS.load(Ordering::Relaxed),
            alloc_bytes: ALLOC_BYTES.load(Ordering::Relaxed),
            dealloc_bytes: DEALLOC_BYTES.load(Ordering::Relaxed),
            realloc_old_bytes: REALLOC_OLD_BYTES.load(Ordering::Relaxed),
            realloc_new_bytes: REALLOC_NEW_BYTES.load(Ordering::Relaxed),
        }
    }

    fn add(self, other: Self) -> Self {
        Self {
            alloc_calls: self.alloc_calls + other.alloc_calls,
            dealloc_calls: self.dealloc_calls + other.dealloc_calls,
            realloc_calls: self.realloc_calls + other.realloc_calls,
            alloc_bytes: self.alloc_bytes + other.alloc_bytes,
            dealloc_bytes: self.dealloc_bytes + other.dealloc_bytes,
            realloc_old_bytes: self.realloc_old_bytes + other.realloc_old_bytes,
            realloc_new_bytes: self.realloc_new_bytes + other.realloc_new_bytes,
        }
    }

    fn div(self, n: u64) -> Self {
        Self {
            alloc_calls: self.alloc_calls / n,
            dealloc_calls: self.dealloc_calls / n,
            realloc_calls: self.realloc_calls / n,
            alloc_bytes: self.alloc_bytes / n,
            dealloc_bytes: self.dealloc_bytes / n,
            realloc_old_bytes: self.realloc_old_bytes / n,
            realloc_new_bytes: self.realloc_new_bytes / n,
        }
    }

    fn gross_allocated_bytes(&self) -> u64 {
        self.alloc_bytes + self.realloc_new_bytes
    }

    fn gross_released_bytes(&self) -> u64 {
        self.dealloc_bytes + self.realloc_old_bytes
    }
}

fn average_duration_and_allocs<F, T>(rounds: u32, mut run: F) -> (Duration, AllocationStats, T)
where
    F: FnMut() -> T,
{
    let mut total = Duration::ZERO;
    let mut total_allocs = AllocationStats::default();
    let mut last_result = None;

    for _ in 0..rounds {
        AllocationStats::reset();
        let start = Instant::now();
        let result = run();
        total += start.elapsed();
        total_allocs = total_allocs.add(AllocationStats::read());
        last_result = Some(result);
    }

    (
        total / rounds,
        total_allocs.div(rounds as u64),
        last_result.expect("rounds must be > 0"),
    )
}

fn main() {
    let iterations = 10000;
    let threads = 4;
    let num_cities = 200;
    let seed = 42;
    let rounds = 1;

    let locations: Vec<_> = locations(num_cities);

    // Warm up both paths to avoid one-time initialization costs skewing results.
    black_box(run_search_parallel_immutable(
        &locations, iterations, seed, threads,
    ));
    black_box(run_search_parallel_use_mut(
        &locations, iterations, seed, threads,
    ));

    let (immutable_avg, immutable_allocs, immutable_result) =
        average_duration_and_allocs(rounds, || {
            run_search_parallel_immutable(&locations, iterations, seed, threads)
        });
    let (use_vec_avg, use_vec_allocs, use_vec_result) = average_duration_and_allocs(rounds, || {
        run_search_parallel_use_mut(&locations, iterations, seed, threads)
    });

    let immutable_best = immutable_result
        .as_ref()
        .map(|(_, distance)| *distance)
        .unwrap_or(f64::NAN);
    let use_vec_best = use_vec_result
        .as_ref()
        .map(|(_, distance)| *distance)
        .unwrap_or(f64::NAN);

    println!(
        "iterations: {iterations}, threads: {threads}, cities: {num_cities}, rounds: {rounds}"
    );
    println!("immutable avg: {:.3} ms", immutable_avg.as_secs_f64() * 1e3);
    println!("use_vec avg:   {:.3} ms", use_vec_avg.as_secs_f64() * 1e3);
    println!(
        "immutable avg allocations: {} calls, {} bytes (gross)",
        immutable_allocs.alloc_calls,
        immutable_allocs.gross_allocated_bytes()
    );
    println!(
        "use_vec avg allocations:   {} calls, {} bytes (gross)",
        use_vec_allocs.alloc_calls,
        use_vec_allocs.gross_allocated_bytes()
    );
    println!(
        "immutable avg releases: {} calls, {} bytes (gross)",
        immutable_allocs.dealloc_calls + immutable_allocs.realloc_calls,
        immutable_allocs.gross_released_bytes()
    );
    println!(
        "use_vec avg releases:   {} calls, {} bytes (gross)",
        use_vec_allocs.dealloc_calls + use_vec_allocs.realloc_calls,
        use_vec_allocs.gross_released_bytes()
    );

    let speedup = immutable_avg.as_secs_f64() / use_vec_avg.as_secs_f64();
    println!("use_vec speedup vs immutable: {speedup:.2}x");

    println!("immutable best distance: {immutable_best:.6}");
    println!("use_vec best distance:   {use_vec_best:.6}");

    if (immutable_best - use_vec_best).abs() > 1e-9 {
        println!(
            "warning: best distances differ by {:.6}",
            (immutable_best - use_vec_best).abs()
        );
    }
}
