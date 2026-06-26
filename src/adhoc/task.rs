/// A unit of work that can be executed and returns a result.
///
/// The `Task` trait is the foundation of zero-cost parallel task composition.
/// Tasks are `Send` to enable execution on thread pools.
pub trait Task: Send {
    /// The type of result produced by executing this task.
    type Output: Send;

    /// Execute the task and return its output.
    fn run(self) -> Self::Output;
}

// output

trait Oq {
    type Push<O>: Oq;
}

struct Os<F> {
    f: F,
}

impl<F> Oq for Os<F> {
    type Push<O> = Om<F, Os<O>>;
}

struct Om<F, B: Oq> {
    f: F,
    b: B,
}

impl<F, B: Oq> Oq for Om<F, B> {
    type Push<O> = Om<F, B::Push<O>>;
}

// task

trait Tq {
    type O: Oq;

    type Push<T: Task>: Tq<O = <Self::O as Oq>::Push<T::Output>>;

    fn push<T: Task>(self, t: T) -> Self::Push<T>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abc() {
        fn xyz<Q, T>(q: Q, t: T)
        where
            Q: Tq<O = usize>,
            T: Task<Output = char>,
        {
            let q2 = q.push(t);
        }
    }
}
