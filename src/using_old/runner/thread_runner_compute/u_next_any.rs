use crate::{
    ThreadRunner,
    generic_values::Values,
    generic_values::runner_results::{Fallibility, Next},
};
use orx_concurrent_iter::{ChunkPuller, ConcurrentIter};

pub fn u_m<C, U, I, O, M1>(
    mut runner: C,
    mut u: U,
    iter: &I,
    shared_state: &C::SharedState,
    map1: &M1,
) -> Option<O>
where
    C: ThreadRunner,
    I: ConcurrentIter,
    O: Send,
    M1: Fn(&mut U, I::Item) -> O,
{
    let u = &mut u;
    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller();

    loop {
        let chunk_size = runner.next_chunk_size(shared_state, iter);

        runner.begin_chunk(chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some(i) => {
                    let first = map1(u, i);
                    iter.skip_to_end();
                    runner.complete_chunk(shared_state, chunk_size);
                    runner.complete_task(shared_state);
                    return Some(first);
                }
                None => break,
            },
            c => {
                if c > chunk_puller.chunk_size() {
                    chunk_puller = iter.chunk_puller(c);
                }

                match chunk_puller.pull() {
                    Some(mut chunk) => {
                        if let Some(i) = chunk.next() {
                            let first = map1(u, i);
                            iter.skip_to_end();
                            runner.complete_chunk(shared_state, chunk_size);
                            runner.complete_task(shared_state);
                            return Some(first);
                        }
                    }
                    None => break,
                }
            }
        }

        runner.complete_chunk(shared_state, chunk_size);
    }

    runner.complete_task(shared_state);
    None
}

pub fn u_x<C, U, I, Vo, X1>(
    mut runner: C,
    mut u: U,
    iter: &I,
    shared_state: &C::SharedState,
    xap1: &X1,
) -> Result<Option<Vo::Item>, <Vo::Fallibility as Fallibility>::Error>
where
    C: ThreadRunner,
    I: ConcurrentIter,
    Vo: Values,
    Vo::Item: Send,
    X1: Fn(&mut U, I::Item) -> Vo,
{
    let u = &mut u;
    let mut chunk_puller = iter.chunk_puller(0);
    let mut item_puller = iter.item_puller();

    loop {
        let chunk_size = runner.next_chunk_size(shared_state, iter);

        runner.begin_chunk(chunk_size);

        match chunk_size {
            0 | 1 => match item_puller.next() {
                Some(i) => {
                    let vt = xap1(u, i);
                    match vt.next() {
                        Next::Done { value } => {
                            if let Some(value) = value {
                                iter.skip_to_end();
                                runner.complete_chunk(shared_state, chunk_size);
                                runner.complete_task(shared_state);
                                return Ok(Some(value));
                            }
                        }
                        Next::StoppedByError { error } => {
                            iter.skip_to_end();
                            runner.complete_chunk(shared_state, chunk_size);
                            runner.complete_task(shared_state);
                            return Err(error);
                        }
                        Next::StoppedByWhileCondition => {
                            iter.skip_to_end();
                            runner.complete_chunk(shared_state, chunk_size);
                            runner.complete_task(shared_state);
                            return Ok(None);
                        }
                    }
                }
                None => break,
            },
            c => {
                if c > chunk_puller.chunk_size() {
                    chunk_puller = iter.chunk_puller(c);
                }

                match chunk_puller.pull() {
                    Some(chunk) => {
                        for i in chunk {
                            let vt = xap1(u, i);
                            match vt.next() {
                                Next::Done { value } => {
                                    if let Some(value) = value {
                                        iter.skip_to_end();
                                        runner.complete_chunk(shared_state, chunk_size);
                                        runner.complete_task(shared_state);
                                        return Ok(Some(value));
                                    }
                                }
                                Next::StoppedByError { error } => {
                                    iter.skip_to_end();
                                    runner.complete_chunk(shared_state, chunk_size);
                                    runner.complete_task(shared_state);
                                    return Err(error);
                                }
                                Next::StoppedByWhileCondition => {
                                    iter.skip_to_end();
                                    runner.complete_chunk(shared_state, chunk_size);
                                    runner.complete_task(shared_state);
                                    return Ok(None);
                                }
                            }
                        }
                    }
                    None => break,
                }
            }
        }

        runner.complete_chunk(shared_state, chunk_size);
    }

    runner.complete_task(shared_state);
    Ok(None)
}
