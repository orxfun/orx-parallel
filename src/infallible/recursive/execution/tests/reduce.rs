use crate::infallible::recursive::execution::reduce::reduce;
use crate::infallible::recursive::execution::tests::tree::{Node, flatten};
use crate::infallible::{Xap, xap_variants::Id};
use crate::{Params, Runner};
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

    let expected = flatten([&tree], &|x: &&Node| &x.children)
        .into_iter()
        .flat_map(|x| xap.xap(x))
        .reduce(|a, b| a + b);
    let result = reduce(runner, params, [&tree], xap, |x| &x.children, |a, b| a + b);

    assert_eq!(result, expected);
}
