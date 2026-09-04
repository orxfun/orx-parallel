use crate::Params;
use crate::infallible::recursive::execution::fold::fold;
use crate::infallible::recursive::execution::tests::tree::{Node, flatten};
use crate::infallible::{Xap, xap_variants::Id};
use crate::runner::default_runner;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use test_case::test_matrix;

#[test_matrix([2, 3, 4], [2, 3, 4])]
fn recursive_fold(depth: usize, fan_out: usize) {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let tree = Node::build_tree(depth, fan_out, &mut rng);

    let xap = Id::<&Node>::new()
        .map(|x| x.value)
        .filter(|x| !x.is_multiple_of(7));

    let runner = default_runner();
    let params = Params::default();

    let expected: u64 = flatten([&tree], &|x: &&Node| &x.children)
        .into_iter()
        .flat_map(|x| xap.xap(x))
        .sum();

    let result = fold(
        runner,
        params,
        [&tree],
        xap,
        |x| &x.children,
        || 0u64,
        |a, b| *a += b,
    );

    let sum: u64 = result.iter().sum();
    assert_eq!(sum, expected);
}
