use crate::{Params, runner::ParRunner};
use alloc::vec::Vec;
use orx_iterable::Iterable;
use std::collections::{BTreeMap, BTreeSet};
use std::{println, time::Instant};

pub struct WithDiagnostics<R: ParRunner>(R);

impl<R: ParRunner> WithDiagnostics<R> {
    pub fn new(inner: R) -> Self {
        Self(inner)
    }
}

impl<R: ParRunner> ParRunner for WithDiagnostics<R> {
    type Pool = R::Pool;

    type State = StateWithDiagnostics<R::State>;

    type ChunkState = ChunkStateWithDiagnostics<R::ChunkState>;

    fn pool(&self) -> &Self::Pool {
        self.0.pool()
    }

    fn pool_mut(&mut self) -> &mut Self::Pool {
        self.0.pool_mut()
    }

    fn with_pool<Q: crate::pool::ParThreadPool>(
        self,
        pool: Q,
    ) -> impl ParRunner<State = Self::State, ChunkState = Self::ChunkState, Pool = Q> {
        self.0.with_pool(pool).with_diagnostics()
    }

    fn do_spawn_new(spawned: usize, state: &Self::State) -> Option<usize> {
        R::do_spawn_new(spawned, &state.inner)
    }

    fn new_state(
        &mut self,
        params: Params,
        max_num_threads: usize,
        initial_len: Option<usize>,
    ) -> Self::State {
        let inner = self.0.new_state(params, max_num_threads, initial_len);
        StateWithDiagnostics::new(max_num_threads, inner)
    }

    fn begin_thread(state: &Self::State, th_idx: usize) {
        state.thread_lifetimes.begin(th_idx);
        R::begin_thread(&state.inner, th_idx);
    }

    fn next_chunk_size(state: &Self::State, remaining: Option<usize>) -> usize {
        R::next_chunk_size(&state.inner, remaining)
    }

    fn begin_chunk(th_idx: usize, chunk_size: usize) -> Self::ChunkState {
        let inner = R::begin_chunk(th_idx, chunk_size);
        ChunkStateWithDiagnostics {
            inner,
            th_idx,
            chunk_size,
        }
    }

    fn complete_chunk(state: &Self::State, chunk_state: Self::ChunkState) {
        state
            .task_counts
            .push(chunk_state.th_idx, chunk_state.chunk_size);
        R::complete_chunk(&state.inner, chunk_state.inner);
    }

    fn complete_thread(state: &Self::State, th_idx: usize) {
        state.thread_lifetimes.end(th_idx);
        R::complete_thread(&state.inner, th_idx);
    }

    fn complete_computation(state: Self::State) {
        state.display();
        R::complete_computation(state.inner);
    }
}

struct TaskCounts(Vec<Vec<usize>>);

impl TaskCounts {
    fn new(max_num_threads: usize) -> Self {
        Self((0..max_num_threads).map(|_| Vec::new()).collect())
    }

    fn push(&self, th_idx: usize, chunk_size: usize) {
        let vec = unsafe { &mut *(self.0.as_ptr().add(th_idx) as *mut Vec<usize>) };
        vec.push(chunk_size);
    }
}

struct ThreadLifetimes {
    begin: Vec<Option<Instant>>,
    end: Vec<Option<Instant>>,
}

impl ThreadLifetimes {
    fn new(max_num_threads: usize) -> Self {
        Self {
            begin: (0..max_num_threads).map(|_| None).collect(),
            end: (0..max_num_threads).map(|_| None).collect(),
        }
    }

    fn begin(&self, th_idx: usize) {
        let elem = unsafe { &mut *(self.begin.as_ptr().add(th_idx) as *mut Option<Instant>) };
        *elem = Some(Instant::now());
    }

    fn end(&self, th_idx: usize) {
        let elem = unsafe { &mut *(self.end.as_ptr().add(th_idx) as *mut Option<Instant>) };
        *elem = Some(Instant::now());
    }
}

pub struct StateWithDiagnostics<S> {
    inner: S,
    task_counts: TaskCounts,
    thread_lifetimes: ThreadLifetimes,
}

impl<S> StateWithDiagnostics<S> {
    pub fn new(max_num_threads: usize, inner: S) -> Self {
        Self {
            inner,
            task_counts: TaskCounts::new(max_num_threads),
            thread_lifetimes: ThreadLifetimes::new(max_num_threads),
        }
    }

    fn display(&self) {
        const NUM_BLOCKS: usize = 60;
        const SEP: &str = "│";
        const BLOCK: &str = "▇";

        let begin = &self.thread_lifetimes.begin;
        let end = &self.thread_lifetimes.end;
        let counts = &self.task_counts.0;

        let used_threads: BTreeSet<_> = counts
            .iter()
            .enumerate()
            .filter(|(_, x)| x.iter().sum::<usize>() > 0)
            .map(|(t, _)| t)
            .collect();

        // Overall wall time: first thread start → last thread end.
        let wall_start = begin.iter().filter_map(|x| *x).min();
        let wall_end = end.iter().filter_map(|x| *x).max();
        let wall_time_ms = match (wall_start, wall_end) {
            (Some(s), Some(e)) => (e - s).as_secs_f64() * 1_000.0,
            _ => 0.0,
        };
        let wall_time_ns = match (wall_start, wall_end) {
            (Some(s), Some(e)) => (e - s).as_nanos().max(1),
            _ => 1,
        };

        // ── Header ──────────────────────────────────────────────────────────
        println!("\n{SEP}");
        println!("{SEP} # Parallel Executor Diagnostics");
        println!("{SEP}");
        println!("{SEP}   Available threads : {}", counts.len());
        println!("{SEP}   Used threads      : {}", used_threads.len());
        println!("{SEP}   Wall time         : {wall_time_ms:.2} ms");

        // ── Summary Table ───────────────────────────────────────────────────
        println!("{SEP}");
        println!("{SEP} ## Summary Table");
        println!(
            "{SEP}   {:>6}  {:>10}  {:>10}  {:>9}  {:>9}  {:>9}  {:>7}",
            "thread", "num_chunks", "num_tasks", "min_chunk", "avg_chunk", "max_chunk", "util%"
        );
        println!(
            "{SEP}   {:->6}  {:->10}  {:->10}  {:->9}  {:->9}  {:->9}  {:->7}",
            "", "", "", "", "", "", ""
        );

        // Per-thread stats collected for balance metrics below.
        let mut thread_tasks: Vec<usize> = Vec::new();

        for (t, task_counts) in counts.iter().enumerate() {
            if used_threads.contains(&t) {
                let total: usize = task_counts.iter().sum();
                let num_chunks = task_counts.len();
                let min_chunk = task_counts.iter().copied().min().unwrap_or(0);
                let max_chunk = task_counts.iter().copied().max().unwrap_or(0);
                let avg_chunk = if num_chunks == 0 {
                    0
                } else {
                    total / num_chunks
                };

                let uptime_ns = match (begin[t], end[t]) {
                    (Some(b), Some(e)) => (e - b).as_nanos(),
                    _ => 0,
                };
                let util = uptime_ns as f64 / wall_time_ns as f64 * 100.0;

                thread_tasks.push(total);
                println!(
                    "{SEP}   {:>6}  {:>10}  {:>10}  {:>9}  {:>9}  {:>9}  {:>6.1}%",
                    t, num_chunks, total, min_chunk, avg_chunk, max_chunk, util
                );
            }
        }

        // ── Workload Balance ────────────────────────────────────────────────
        // These two scalars give a quick read on how evenly work was shared:
        //   - max/min ratio: 1.00 means perfect balance; higher values indicate
        //     that some threads processed significantly more items than others.
        //   - coefficient of variation (CV): standard deviation expressed as a
        //     percentage of the mean; 0% is perfect balance, lower is better.
        if thread_tasks.len() > 1 {
            let max_t = *thread_tasks.iter().max().unwrap_or(&1) as f64;
            let min_t = *thread_tasks.iter().min().unwrap_or(&1) as f64;
            let mean = thread_tasks.iter().sum::<usize>() as f64 / thread_tasks.len() as f64;
            let variance = thread_tasks
                .iter()
                .map(|&t| {
                    let d = t as f64 - mean;
                    d * d
                })
                .sum::<f64>()
                / thread_tasks.len() as f64;
            let cv = if mean > 0.0 {
                variance.sqrt() / mean * 100.0
            } else {
                0.0
            };

            println!("{SEP}");
            println!("{SEP} ## Workload Balance");
            println!(
                "{SEP}   max/min task ratio  : {:.2}x  (1.00 = perfect balance)",
                max_t / min_t.max(1.0)
            );
            println!("{SEP}   coeff. of variation : {cv:.1}%  (lower is better)");
        }

        // ── Thread Active Timeline ───────────────────────────────────────────
        // Each bar shows when the thread started (offset from the first thread)
        // and how long it was active. Threads that start later appear indented.
        struct ThreadLife {
            beg: Instant,
            end: Instant,
            beg_ns: u128,
        }
        impl ThreadLife {
            fn up_time_ns(&self) -> u128 {
                (self.end - self.beg).as_nanos()
            }
        }

        let mut threads = BTreeMap::new();
        for (t, (beg, end)) in begin.iter().copied().zip(end.iter().copied()).enumerate() {
            if used_threads.contains(&t) {
                if let (Some(beg), Some(end)) = (beg, end) {
                    threads.insert(
                        t,
                        ThreadLife {
                            beg,
                            end,
                            beg_ns: 0,
                        },
                    );
                }
            }
        }

        let max_up_time_ns = threads
            .values()
            .map(|x| x.up_time_ns())
            .max()
            .unwrap_or_default();
        let block_len_ns = std::cmp::max(1, max_up_time_ns / NUM_BLOCKS as u128);

        if let Some(min_beg) = threads.values().map(|x| x.beg).min() {
            for x in threads.values_mut() {
                x.beg_ns = (x.beg - min_beg).as_nanos();
            }
        }

        println!("{SEP}");
        println!(
            "{SEP} ## Thread Active Timeline  (each block ≈ {:.2} ms)",
            block_len_ns as f64 / 1_000_000.0
        );
        for (t, life) in threads.iter() {
            let offset = (life.beg_ns / block_len_ns) as usize;
            let busy = std::cmp::max(1, (life.up_time_ns() / block_len_ns) as usize);
            println!(
                "{SEP}   [{t:>2}] {}{}",
                " ".repeat(offset),
                BLOCK.repeat(busy)
            );
        }

        // ── Thread Task Distribution ────────────────────────────────────────
        // Bar length is proportional to the number of tasks the thread processed.
        // Compare bar lengths to spot which threads carried a heavier share.
        let max_num_tasks = counts
            .iter()
            .enumerate()
            .filter(|(t, _)| used_threads.contains(t))
            .map(|(_, x)| x.iter().sum::<usize>())
            .max()
            .unwrap_or(0);
        let task_block_len = std::cmp::max(1, max_num_tasks / NUM_BLOCKS);

        println!("{SEP}");
        println!("{SEP} ## Thread Task Distribution  (bar length ∝ tasks processed)");
        for (t, task_counts) in counts
            .iter()
            .enumerate()
            .filter(|(t, _)| used_threads.contains(t))
        {
            let num_tasks = task_counts.iter().sum::<usize>();
            let bars = std::cmp::max(1, num_tasks / task_block_len);
            println!("{SEP}   [{t:>2}] {}  ({})", BLOCK.repeat(bars), num_tasks);
        }

        println!("{SEP}");
        println!();
    }
}

pub struct ChunkStateWithDiagnostics<C> {
    inner: C,
    th_idx: usize,
    chunk_size: usize,
}
