use crate::parameters::{ChunkSize, Params};
use crate::pool::ParThreadPool;
use crate::runner::par_runner::ParRunner;
#[cfg(feature = "std")]
use crate::runner::runner_variants::run_b::state::Mode;
use crate::runner::runner_variants::run_b::state::{ChunkState, State};
use core::cmp::min;

#[cfg(feature = "std")]
std::thread_local! {
    static ENABLE_RUNNER_B_DIAGNOSTICS: std::cell::Cell<bool> = std::cell::Cell::new(false);
    static LAST_RUNNER_B_DIAGNOSTICS: std::cell::RefCell<Option<crate::runner::runner_variants::run_b::state::DiagnosticData>> = std::cell::RefCell::new(None);
}

#[cfg(feature = "std")]
/// Enable or disable diagnostics collection for RunnerB exploration phase.
/// When enabled, RunnerB will collect chunk size and elapsed time samples during exploration.
/// This is used for analyzing adaptive scheduling behavior.
pub fn enable_runner_b_diagnostics(enable: bool) {
    ENABLE_RUNNER_B_DIAGNOSTICS.with(|flag| flag.set(enable));
}

#[cfg(feature = "std")]
/// Retrieve diagnostics from the last RunnerB parallel computation on this thread.
/// Returns `None` if diagnostics were not enabled or no computation has run yet.
pub fn take_last_runner_b_diagnostics()
-> Option<crate::runner::runner_variants::run_b::state::DiagnosticData> {
    LAST_RUNNER_B_DIAGNOSTICS.with(|cell| cell.borrow_mut().take())
}

#[cfg(feature = "std")]
fn should_collect_diagnostics() -> bool {
    ENABLE_RUNNER_B_DIAGNOSTICS.with(|flag| flag.get())
}

#[cfg(not(feature = "std"))]
fn should_collect_diagnostics() -> bool {
    false
}

pub struct RunnerB<P: ParThreadPool> {
    pool: P,
}

unsafe impl<P: ParThreadPool> Sync for RunnerB<P> {}

impl<P: ParThreadPool> RunnerB<P> {
    pub fn new(pool: P) -> Self {
        Self { pool }
    }
}

impl<P: ParThreadPool> ParRunner for RunnerB<P> {
    type Pool = P;

    type State = State;

    type ChunkState = crate::runner::runner_variants::run_b::state::ChunkState;

    fn pool(&self) -> &Self::Pool {
        &self.pool
    }

    fn pool_mut(&mut self) -> &mut Self::Pool {
        &mut self.pool
    }

    fn with_pool<Q: ParThreadPool>(
        self,
        pool: Q,
    ) -> impl ParRunner<State = Self::State, ChunkState = Self::ChunkState, Pool = Q> {
        RunnerB::new(pool)
    }

    fn do_spawn_new(spawned: usize, state: &Self::State) -> Option<usize> {
        (spawned < state.max_num_threads).then_some(spawned)
    }

    fn new_state(
        &mut self,
        params: Params,
        max_num_threads: usize,
        _size_hint: (usize, Option<usize>),
    ) -> Self::State {
        let min_chunk_size = match params.chunk_size {
            ChunkSize::Auto => 1,
            ChunkSize::Min(chunk_size) | ChunkSize::Exact(chunk_size) => chunk_size.into(),
        };

        let fixed_chunk_size = match params.chunk_size {
            ChunkSize::Exact(chunk_size) => Some(chunk_size.into()),
            _ => None,
        };

        State {
            max_num_threads,
            min_chunk_size,
            fixed_chunk_size,
            #[cfg(feature = "std")]
            initial_len: match _size_hint.1 {
                Some(upper_bound) if upper_bound == _size_hint.0 => Some(upper_bound),
                _ => None,
            },
            #[cfg(feature = "std")]
            explore_started_at: std::time::Instant::now(),
            #[cfg(feature = "std")]
            mode: core::sync::atomic::AtomicUsize::new(0),
            #[cfg(feature = "std")]
            chosen_chunk_size: core::sync::atomic::AtomicUsize::new(fixed_chunk_size.unwrap_or(0)),
            #[cfg(feature = "std")]
            explored_tasks: core::sync::atomic::AtomicUsize::new(0),
            #[cfg(feature = "std")]
            avg_ns_per_item: core::sync::atomic::AtomicU64::new(0),
            #[cfg(feature = "std")]
            avg_abs_deviation_ns_per_item: core::sync::atomic::AtomicU64::new(0),
            #[cfg(feature = "std")]
            prev_avg_ns_per_item: core::sync::atomic::AtomicU64::new(0),
            #[cfg(feature = "std")]
            converged_samples: core::sync::atomic::AtomicUsize::new(0),
            #[cfg(feature = "std")]
            collect_diagnostics: should_collect_diagnostics(),
            #[cfg(feature = "std")]
            diagnostics: std::sync::Mutex::new(if should_collect_diagnostics() {
                Some(
                    crate::runner::runner_variants::run_b::state::DiagnosticData {
                        samples: std::vec::Vec::new(),
                        exploration_phase_started_at: std::time::Instant::now(),
                    },
                )
            } else {
                None
            }),
        }
    }

    #[inline(always)]
    fn begin_thread(_: &Self::State, _: usize) {}

    #[inline(always)]
    fn next_chunk_size(state: &Self::State, size_hint: (usize, Option<usize>)) -> usize {
        if let Some(fixed_chunk_size) = state.fixed_chunk_size {
            let remaining = size_hint.1.unwrap_or(size_hint.0).max(1);
            return min(fixed_chunk_size, remaining);
        }

        #[cfg(feature = "std")]
        {
            return match state.mode() {
                Mode::Explore => min(
                    state.min_chunk_size,
                    size_hint.1.unwrap_or(size_hint.0).max(1),
                ),
                Mode::Fixed => state.selected_chunk_size(size_hint),
            };
        }

        #[cfg(not(feature = "std"))]
        {
            min(
                state.min_chunk_size,
                size_hint.1.unwrap_or(size_hint.0).max(1),
            )
        }
    }

    #[inline(always)]
    fn begin_chunk(_: usize, chunk_size: usize) -> Self::ChunkState {
        ChunkState::new(chunk_size)
    }

    #[inline(always)]
    fn complete_chunk(state: &Self::State, chunk_state: Self::ChunkState) {
        #[cfg(feature = "std")]
        {
            if state.mode() == Mode::Explore {
                state.record_chunk(chunk_state);
                if state.should_stop_exploration() {
                    state.complete_exploration();
                }
            }
        }

        #[cfg(not(feature = "std"))]
        {
            let _ = (state, chunk_state);
        }
    }

    #[inline(always)]
    fn complete_thread(_: &Self::State, _: usize) {}

    #[inline(always)]
    fn complete_computation(state: Self::State) {
        #[cfg(feature = "std")]
        if state.collect_diagnostics {
            if let Some(diag) = state.take_diagnostics() {
                LAST_RUNNER_B_DIAGNOSTICS.with(|cell| {
                    *cell.borrow_mut() = Some(diag);
                });
            }
        }
    }
}
