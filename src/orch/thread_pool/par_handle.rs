use std::any::Any;

pub type JoinResult<T> = Result<T, Box<dyn Any + Send + 'static>>;

pub trait ParHandle<'scope, T> {
    fn join(self) -> JoinResult<T>;

    fn is_finished(&self) -> bool;
}
