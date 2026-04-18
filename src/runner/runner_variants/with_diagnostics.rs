use crate::{Params, runner::ParRunner};
use alloc::vec::Vec;
use orx_iterable::Iterable;
use std::collections::{BTreeMap, BTreeSet};
use std::{println, time::Instant};

pub struct RunnerWithDiagnostics<R: ParRunner>(R);

impl<R: ParRunner> RunnerWithDiagnostics<R> {
    pub fn new(inner: R) -> Self {
        Self(inner)
    }
}

impl<R: ParRunner> ParRunner for RunnerWithDiagnostics<R> {
    type Pool = R::Pool;

    type State = StateWithDiagnostics<R::State>;

    type ChunkState = ChunkStateWithDiagnostics<R::ChunkState>;

    fn pool(&self) -> &Self::Pool {
        self.0.pool()
    }

    fn pool_mut(&mut self) -> &mut Self::Pool {
        self.0.pool_mut()
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
        const NUM_BLOCKS: usize = 70;
        const LINE: &'static str = "│";
        const BLOCK: &'static str = "▇";

        let begin = &self.thread_lifetimes.begin;
        let end = &self.thread_lifetimes.end;

        let counts = &self.task_counts.0;
        let used_threads: BTreeSet<_> = counts
            .iter()
            .enumerate()
            .filter(|(_, x)| x.iter().sum::<usize>() > 0)
            .map(|(t, _)| t)
            .collect();

        println!("\n# Parallel Executor Diagnostics");
        println!("{LINE}\n{LINE}");
        println!("{LINE} - Number of available threads = {}", counts.len());
        println!(
            "{LINE} - Number of actually used threads = {}",
            used_threads.len()
        );

        println!("{LINE}\n{LINE}\n{LINE} ## Summary Table");
        println!("{LINE} [Thread idx]: num_calls, num_tasks, avg_chunk_size");

        for (t, task_counts) in counts.iter().enumerate() {
            if used_threads.contains(&t) {
                let total: usize = task_counts.iter().sum();
                let num_calls = task_counts.len();
                let avg_chunk_size = match num_calls {
                    0 => 0,
                    n => total / n,
                };

                println!("{LINE} [{t}]:\t{num_calls}\t{total}\t{avg_chunk_size}",);
            }
        }

        println!("{LINE}\n{LINE}\n{LINE} ## Thread Active Timeline");
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
            if used_threads.contains(&t)
                && let (Some(beg), Some(end)) = (beg, end)
            {
                let beg_ns = 0;
                threads.insert(t, ThreadLife { beg, end, beg_ns });
            }
        }
        let max_up_time_ns = threads
            .values()
            .map(|x| x.up_time_ns())
            .max()
            .unwrap_or_default();
        let block_len = std::cmp::max(1, max_up_time_ns / NUM_BLOCKS as u128);

        if let Some(min_beg) = threads.values().map(|x| x.beg).min() {
            for x in threads.values_mut() {
                x.beg_ns = (x.beg - min_beg).as_nanos();
            }
        }

        for (t, life) in threads.iter() {
            let beg = (life.beg_ns / block_len) as usize;
            let busy = (life.up_time_ns() / block_len) as usize;
            println!("{LINE} [{t}]:\t{}{}", "".repeat(beg), BLOCK.repeat(busy));
        }

        println!("{LINE}\n{LINE}\n{LINE} ## Thread Task Counts");

        let max_num_tasks = counts
            .iter()
            .enumerate()
            .filter(|(t, _)| used_threads.contains(&t))
            .map(|(_, x)| x.iter().sum::<usize>())
            .max()
            .unwrap_or(0);
        let block_len = std::cmp::max(1, max_num_tasks / NUM_BLOCKS);
        for (t, counts) in counts
            .iter()
            .enumerate()
            .filter(|(t, _)| used_threads.contains(&t))
        {
            let num_tasks = counts.iter().sum::<usize>() / block_len;
            println!("{LINE} [{t}]:\t{}", BLOCK.repeat(num_tasks));
        }

        println!();
    }
}

pub struct ChunkStateWithDiagnostics<C> {
    inner: C,
    th_idx: usize,
    chunk_size: usize,
}
