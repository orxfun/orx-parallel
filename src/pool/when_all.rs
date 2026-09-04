use crate::Scope;
use core::marker::PhantomData;

pub trait WhenAll {
    type PushBack<Elem>: WhenAll
    where
        Elem: FnOnce() + Send;

    type Front: FnOnce();

    type Back: WhenAll;

    fn push<Elem>(self, element: Elem) -> Self::PushBack<Elem>
    where
        Elem: FnOnce() + Send;

    fn run(self);
}

// single

pub struct WhenAllSingle<'s, 'env, 'scope, S, F>
where
    S: Scope<'s, 'env, 'scope>,
    F: FnOnce() + Send,
{
    scope: S,
    front: F,
    p: PhantomData<&'s &'env &'scope ()>,
}

impl<'s, 'env, 'scope, S, F> WhenAllSingle<'s, 'env, 'scope, S, F>
where
    S: Scope<'s, 'env, 'scope>,
    F: FnOnce() + Send,
{
    pub fn new(scope: S, front: F) -> Self {
        Self {
            scope,
            front,
            p: PhantomData,
        }
    }
}

impl<'s, 'env, 'scope, S, F> WhenAll for WhenAllSingle<'s, 'env, 'scope, S, F>
where
    S: Scope<'s, 'env, 'scope>,
    F: FnOnce() + Send,
{
    type PushBack<Elem>
        = WhenAllPair<'s, 'env, 'scope, S, F, WhenAllSingle<'s, 'env, 'scope, S, Elem>>
    where
        Elem: FnOnce() + Send;

    type Front = F;

    type Back = Self;

    fn push<Elem>(self, element: Elem) -> Self::PushBack<Elem>
    where
        Elem: FnOnce() + Send,
    {
        let back = WhenAllSingle::new(self.scope, element);
        WhenAllPair::new(self.scope, self.front, back)
    }

    fn run(self) {
        let (scope, work) = (self.scope, self.front);
        scope.run(work);
    }
}

// pair

pub struct WhenAllPair<'s, 'env, 'scope, S, F, B>
where
    S: Scope<'s, 'env, 'scope>,
    F: FnOnce() + Send,
    B: WhenAll,
{
    scope: S,
    front: F,
    back: B,
    p: PhantomData<&'s &'env &'scope ()>,
}

impl<'s, 'env, 'scope, S, F, B> WhenAllPair<'s, 'env, 'scope, S, F, B>
where
    S: Scope<'s, 'env, 'scope>,
    F: FnOnce() + Send,
    B: WhenAll,
{
    pub fn new(scope: S, front: F, back: B) -> Self {
        Self {
            scope,
            front,
            back,
            p: PhantomData,
        }
    }
}

impl<'s, 'env, 'scope, S, F, B> WhenAll for WhenAllPair<'s, 'env, 'scope, S, F, B>
where
    S: Scope<'s, 'env, 'scope>,
    F: FnOnce() + Send,
    B: WhenAll,
{
    type PushBack<Elem>
        = WhenAllPair<'s, 'env, 'scope, S, F, B::PushBack<Elem>>
    where
        Elem: FnOnce() + Send;

    type Front = F;

    type Back = B;

    fn push<Elem>(self, element: Elem) -> Self::PushBack<Elem>
    where
        Elem: FnOnce() + Send,
    {
        let back = self.back.push(element);
        WhenAllPair::new(self.scope, self.front, back)
    }

    fn run(self) {}
}

#[cfg(test)]
#[test]
fn abc() {
    use crate::{ThreadPool, global_pool};
    use std::*;

    global_pool().scope(|s| {
        let a = WhenAllSingle::new(s, || println!("a"))
            .push(|| println!("b"))
            .push(|| println!("c"))
            .push(|| println!("d"));
    });
}
