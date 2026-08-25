use crate::infallible::{Xap, xap_variants::Id};
use crate::recursive::infallible::execution::reduce::reduce;
use crate::recursive::infallible::execution::tests::tree::Node;
use crate::{Params, Runner};
use alloc::vec::Vec;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use test_case::test_matrix;

#[test_matrix([2, 3, 4], [2, 3, 4])]
fn recursive_reduce(depth: usize, fan_out: usize) {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let tree = Node::build_tree(depth, fan_out, &mut rng);

    let xap = Id::<&Node>::new()
        .map(|x| x.value)
        .filter(|x| !x.is_multiple_of(7));

    let runner = Runner::adaptive();
    let params = Params::default();

    let expected = seq_reduce([&tree], &xap, |x| &x.children, |a, b| a + b);
    let result = reduce(runner, params, [&tree], xap, |x| &x.children, |a, b| a + b);

    assert_eq!(result, expected);
}

#[cfg(test)]
pub fn seq_reduce<C, X, F, I, E>(iter: C, xap: &X, extend: E, f: F) -> Option<X::O>
where
    C: IntoIterator,
    X: Xap<I = C::Item>,
    I: IntoIterator<Item = X::I>,
    E: Fn(&X::I) -> I + Send + Sync,
    F: Fn(X::O, X::O) -> X::O + Sync,
{
    fn collect_into<I, E>(
        extend: &E,
        all: &mut Vec<I::Item>,
        iter: impl IntoIterator<Item = I::Item>,
    ) where
        I: IntoIterator,
        E: Fn(&I::Item) -> I + Send + Sync,
    {
        for x in iter {
            let children = extend(&x);
            collect_into(extend, all, children);
            all.push(x);
        }
    }

    let mut all = Vec::<X::I>::new();
    collect_into(&extend, &mut all, iter);
    all.into_iter().flat_map(|x| xap.xap(x)).reduce(f)
}
