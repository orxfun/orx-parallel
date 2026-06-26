pub trait Task {}

orx_meta::define_queue!(
    elements => [Task];
    queue => [Q; Qs, Qm];
);

impl<F: Task> Task for Qs<F> {}

impl<F: Task, B: Q> Task for Qm<F, B> {}

pub struct TaskOf<O, F>
where
    F: FnOnce() -> O,
{
    f: F,
    output: Option<O>,
}

impl<O, F: FnOnce() -> O> TaskOf<O, F> {
    pub fn new(f: F) -> Self {
        Self { f, output: None }
    }
}

impl<O, F: FnOnce() -> O> Task for TaskOf<O, F> {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn abc() {
        let q = Qm::new(TaskOf::new(|| 12))
            .push(TaskOf::new(|| 'x'))
            .push(TaskOf::new(|| true));

        let mut pool = Pool::once(4);
        pool.scoped_computation(|s| {
            //
        });
    }
}
