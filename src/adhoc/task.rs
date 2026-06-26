use crate::ParThreadPool;

pub trait Task {}

orx_meta::define_queue!(
    // lt => [];
    generics => [P:ParThreadPool];
    elements => [Task];
    queue => [Q; Qs, Qm];
    // queue_of => qof;
    // builder => Billy;
);

impl<P: ParThreadPool, F: Task> Task for Qs<P, F> {}

impl<P: ParThreadPool, F: Task, B: Q<P>> Task for Qm<P, F, B> {}

pub struct TaskOf<O> {
    output: Option<O>,
}

impl<O> TaskOf<O> {
    pub fn load<'s, 'env, 'scope, F, P: ParThreadPool>(
        s: &P::ScopeRef<'s, 'env, 'scope>,
        f: F,
    ) -> Self
    where
        F: FnOnce() -> O + Send + 'scope + 'env,
    {
        let x = Self { output: None };
        let f = std::sync::Mutex::new(Some(f));
        P::run_in_scope(s, move || {
            if let Some(func) = f.lock().unwrap().take() {
                let _result = func();
            }
        });
        x
    }
}

// impl<O, F: FnOnce() -> O> Task for TaskOf<O, F> {}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::*;

//     #[test]
//     fn abc() {
//         let q = Qm::new(TaskOf::new(|| 12))
//             .push(TaskOf::new(|| 'x'))
//             .push(TaskOf::new(|| true));

//         let mut pool = Pool::once(4);
//         xyz(&mut pool);
//     }

//     fn xyz<P: ParThreadPool>(pool: &mut P) {
//         pool.scoped_computation(|s| {
//             P::run_in_scope(&s, || {
//                 //
//                 let a = 12;
//             });
//         });
//     }
// }
