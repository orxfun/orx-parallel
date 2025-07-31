use crate::runner::{ParallelTask, ParallelTaskWithIdx, thread_runner::ThreadRunnerCompute};
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

pub fn run<C, I, T>(mut c: C, iter: &I, shared_state: &C::SharedState, mut task: T)
where
    C: ThreadRunnerCompute,
    I: ConcurrentIter,
    T: ParallelTask<Item = I::Item>,
{
    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller();

    loop {
        let chunk_size = c.next_chunk_size(shared_state, iter);

        c.begin_chunk(chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some(value) => task.f1(value),
                None => break,
            },
            c => {
                if c > chunk_puller.chunk_size() {
                    chunk_puller = iter.chunk_puller(c);
                }

                match chunk_puller.pull() {
                    Some(chunk) => task.fc(chunk),
                    None => break,
                }
            }
        }

        c.complete_chunk(shared_state, chunk_size);
    }

    c.complete_task(shared_state);
}

pub fn run_with_idx<C, I, T>(mut c: C, iter: &I, shared_state: &C::SharedState, mut task: T)
where
    C: ThreadRunnerCompute,
    I: ConcurrentIter,
    T: ParallelTaskWithIdx<Item = I::Item>,
{
    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller_with_idx();

    loop {
        let chunk_size = c.next_chunk_size(shared_state, iter);

        c.begin_chunk(chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some((idx, value)) => task.f1(idx, value),
                None => break,
            },
            c => {
                if c > chunk_puller.chunk_size() {
                    chunk_puller = iter.chunk_puller(c);
                }

                match chunk_puller.pull_with_idx() {
                    Some((begin_idx, chunk)) => task.fc(begin_idx, chunk),
                    None => break,
                }
            }
        }

        c.complete_chunk(shared_state, chunk_size);
    }

    c.complete_task(shared_state);
}
