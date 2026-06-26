use crate::adhoc::task::Task;
use orx_meta::queue::*;

pub trait TaskQueue: Task {
    type PushTask<T: Task>: TaskQueue;

    // type PushOutput<T: Task>: StQueue;
}
