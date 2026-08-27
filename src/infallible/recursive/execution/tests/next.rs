use crate::Params;
use crate::infallible::recursive::execution::next::next;
use crate::infallible::recursive::execution::tests::tree::{Node, RANGE};
use crate::infallible::{Xap, xap_variants::Id};
use crate::runner::default_runner;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use test_case::test_matrix;

#[test_matrix([2, 3, 4], [2, 3, 4], [true, false])]
fn recursive_next(depth: usize, fan_out: usize, find: bool) {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let tree = Node::build_tree(depth, fan_out, &mut rng);

    let filter = move |x: &u64| match find {
        true => *x < RANGE.end / 2,
        false => *x > RANGE.end + 1,
    };

    let xap = Id::<&Node>::new().map(|x| x.value).filter(filter);
    let runner = default_runner();
    let params = Params::default();

    let result = next(runner, params, [&tree], xap, |x| &x.children);

    match find {
        true => assert!(result.map(|x| filter(&x)).unwrap_or(true)),
        false => assert!(result.is_none()),
    }
}

#[cfg(not(miri))]
#[test_matrix([7, 8], [8, 9], [true])]
fn recursive_next_determinism(depth: usize, fan_out: usize, find: bool) {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let tree = Node::build_tree(depth, fan_out, &mut rng);

    let filter = move |x: &u64| match find {
        true => *x > 99 * RANGE.end / 100,
        false => *x > RANGE.end + 1,
    };

    let xap = Id::<&Node>::new().map(|x| x.value).filter(filter);
    let mut runner = default_runner();
    let params = Params::default();

    let expected = next(&mut runner, params, [&tree], xap, |x| &x.children);

    for _ in 0..1000 {
        let result = next(&mut runner, params, [&tree], xap, |x| &x.children);
        assert_eq!(expected, result);
    }
}

#[cfg(not(miri))]
#[test_matrix([7, 8], [8, 9], [true])]
fn recursive_next_determinism_with_flat_map(depth: usize, fan_out: usize, find: bool) {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let tree = Node::build_tree(depth, fan_out, &mut rng);

    let filter = move |x: &u64| match find {
        true => *x > 99 * RANGE.end / 100,
        false => *x > RANGE.end + 1,
    };

    let xap = Id::<&Node>::new()
        .flat_map(|x| (0..10).map(|i| x.value + i))
        .filter(filter);
    let mut runner = default_runner();
    let params = Params::default();

    let expected = next(&mut runner, params, [&tree], xap, |x| &x.children);

    for _ in 0..1000 {
        let result = next(&mut runner, params, [&tree], xap, |x| &x.children);
        assert_eq!(expected, result);
    }
}
