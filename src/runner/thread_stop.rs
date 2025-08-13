pub trait ThreadStop {
    type ReduceComputation;

    fn reduce_computation_to(self) -> Option<Self::ReduceComputation>;
}

pub struct ThreadStopByWhile;

impl ThreadStop for ThreadStopByWhile {
    type ReduceComputation = ();

    fn reduce_computation_to(self) -> Option<Self::ReduceComputation> {
        None
    }
}

pub struct ThreadStopByError<E>(E);

impl<E> ThreadStop for ThreadStopByError<E> {
    type ReduceComputation = E;

    fn reduce_computation_to(self) -> Option<Self::ReduceComputation> {
        Some(self.0)
    }
}
