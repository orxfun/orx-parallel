use crate::Scope;
use core::marker::PhantomData;

pub trait WhenAll<'s, 'env, 'scope>
where
    'scope: 's,
    'env: 'scope + 's,
{
    type PushBack<Elem>: WhenAll<'s, 'env, 'scope>
    where
        Elem: FnOnce() + Send + 'scope + 'env;

    type Front: FnOnce() + Send + 'scope + 'env;

    type Back: WhenAll<'s, 'env, 'scope>;

    fn push<Elem>(self, element: Elem) -> Self::PushBack<Elem>
    where
        Elem: FnOnce() + Send + 'scope + 'env;

    fn run(self);
}

// empty

pub struct WhenAllEmpty<'s, 'env, 'scope, S, F>
where
    'scope: 's,
    'env: 'scope + 's,
    S: Scope<'s, 'env, 'scope>,
{
    scope: S,
    p: PhantomData<&'s &'env &'scope F>,
}

impl<'s, 'env, 'scope, S, F> WhenAllEmpty<'s, 'env, 'scope, S, F>
where
    'scope: 's,
    'env: 'scope + 's,
    S: Scope<'s, 'env, 'scope>,
    F: FnOnce() + Send + 'scope + 'env,
{
    pub fn new(scope: S, _do_nothing: F) -> Self {
        Self {
            scope,
            p: PhantomData,
        }
    }
}

impl<'s, 'env, 'scope, S, F> WhenAll<'s, 'env, 'scope> for WhenAllEmpty<'s, 'env, 'scope, S, F>
where
    'scope: 's,
    'env: 'scope + 's,
    S: Scope<'s, 'env, 'scope>,
    F: FnOnce() + Send + 'scope + 'env,
{
    type PushBack<Elem>
        = WhenAllSingle<'s, 'env, 'scope, S, Elem>
    where
        Elem: FnOnce() + Send + 'scope + 'env;

    type Front = F;

    type Back = Self;

    fn push<Elem>(self, element: Elem) -> Self::PushBack<Elem>
    where
        Elem: FnOnce() + Send + 'scope + 'env,
    {
        WhenAllSingle::new(self.scope, element)
    }

    fn run(self) {}
}

// single

pub struct WhenAllSingle<'s, 'env, 'scope, S, F>
where
    'scope: 's,
    'env: 'scope + 's,
    S: Scope<'s, 'env, 'scope>,
    F: FnOnce() + Send + 'scope + 'env,
{
    scope: S,
    front: F,
    p: PhantomData<&'s &'env &'scope ()>,
}

impl<'s, 'env, 'scope, S, F> WhenAllSingle<'s, 'env, 'scope, S, F>
where
    'scope: 's,
    'env: 'scope + 's,
    S: Scope<'s, 'env, 'scope>,
    F: FnOnce() + Send + 'scope + 'env,
{
    pub fn new(scope: S, front: F) -> Self {
        Self {
            scope,
            front,
            p: PhantomData,
        }
    }
}

impl<'s, 'env, 'scope, S, F> WhenAll<'s, 'env, 'scope> for WhenAllSingle<'s, 'env, 'scope, S, F>
where
    'scope: 's,
    'env: 'scope + 's,
    S: Scope<'s, 'env, 'scope>,
    F: FnOnce() + Send + 'scope + 'env,
{
    type PushBack<Elem>
        = WhenAllPair<'s, 'env, 'scope, S, F, WhenAllSingle<'s, 'env, 'scope, S, Elem>>
    where
        Elem: FnOnce() + Send + 'scope + 'env;

    type Front = F;

    type Back = Self;

    fn push<Elem>(self, element: Elem) -> Self::PushBack<Elem>
    where
        Elem: FnOnce() + Send + 'scope + 'env,
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
    'scope: 's,
    'env: 'scope + 's,
    S: Scope<'s, 'env, 'scope>,
    F: FnOnce() + Send + 'scope + 'env,
    B: WhenAll<'s, 'env, 'scope>,
{
    scope: S,
    front: F,
    back: B,
    p: PhantomData<&'s &'env &'scope ()>,
}

impl<'s, 'env, 'scope, S, F, B> WhenAllPair<'s, 'env, 'scope, S, F, B>
where
    'scope: 's,
    'env: 'scope + 's,
    S: Scope<'s, 'env, 'scope>,
    F: FnOnce() + Send + 'scope + 'env,
    B: WhenAll<'s, 'env, 'scope>,
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

impl<'s, 'env, 'scope, S, F, B> WhenAll<'s, 'env, 'scope> for WhenAllPair<'s, 'env, 'scope, S, F, B>
where
    'scope: 's,
    'env: 'scope + 's,
    S: Scope<'s, 'env, 'scope>,
    F: FnOnce() + Send + 'scope + 'env,
    B: WhenAll<'s, 'env, 'scope>,
{
    type PushBack<Elem>
        = WhenAllPair<'s, 'env, 'scope, S, F, B::PushBack<Elem>>
    where
        Elem: FnOnce() + Send + 'scope + 'env;

    type Front = F;

    type Back = B;

    fn push<Elem>(self, element: Elem) -> Self::PushBack<Elem>
    where
        Elem: FnOnce() + Send + 'scope + 'env,
    {
        let back = self.back.push(element);
        WhenAllPair::new(self.scope, self.front, back)
    }

    fn run(self) {
        let (scope, work, remaining) = (self.scope, self.front, self.back);
        scope.run(work);
        remaining.run();
    }
}

#[cfg(test)]
#[test]
fn abc() {
    use crate::{ThreadPool, global_pool};
    use core::num::NonZeroUsize;
    use core::time::Duration;
    use std::*;

    let work_for = |n| std::thread::sleep(Duration::from_millis(n));

    global_pool().scope(|s| {
        let t1 = || {
            work_for(90);
            println!("t1 completes 4th");
        };

        let t2 = || println!("t2 completes 1st");

        let t3 = || {
            work_for(10);
            println!("t3 completes 2nd");
        };

        let t4 = || {
            work_for(50);
            println!("t4 completes 3rd");
        };

        WhenAllSingle::new(s, t1).push(t2).push(t3).push(t4).run();
    });

    assert_eq!(global_pool().max_num_threads(), NonZeroUsize::MAX);
}
