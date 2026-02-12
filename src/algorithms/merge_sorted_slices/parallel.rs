use crate::algorithms::data_structures::Slice;
use orx_concurrent_recursive_iter::{ConcurrentRecursiveIter, Queue};

struct TaskData<'a, T> {
    left: Slice<'a, T>,
    right: Slice<'a, T>,
    target: Slice<'a, T>,
}

impl<'a, T> Clone for TaskData<'a, T> {
    fn clone(&self) -> Self {
        Self {
            left: self.left.clone(),
            right: self.right.clone(),
            target: self.target.clone(),
        }
    }
}

enum TaskKind {
    CopyOneSide,
    SequentialMerge,
    Split,
}

struct Task<'a, T> {
    data: TaskData<'a, T>,
    kind: TaskKind,
}

impl<'a, T> Task<'a, T> {
    fn as_copy_one_side(&self) -> Self {
        Self {
            data: self.data.clone(),
            kind: TaskKind::CopyOneSide,
        }
    }
}

fn extend<'a, 'b, T>(task: &'a Task<'b, T>, queue: &Queue<'b, Task<'b, T>>)
where
    T: Send,
{
    match (task.data.left.len(), task.data.right.len()) {
        (0, _) | (_, 0) => queue.push(task.as_copy_one_side()),
        _ => todo!(),
    }
}
