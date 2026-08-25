use crate::collectables::alg::merge_collected::merge_arb_into_vec;
use crate::infallible::{Xap, xap_variants::Id};
use crate::recursive::infallible::execution::collect_arb::collect_arb;
use crate::recursive::infallible::execution::tests::tree::{Node, flatten};
use crate::{Params, Runner};
use alloc::vec::Vec;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use test_case::test_matrix;

#[test_matrix([2, 3, 4], [2, 3, 4])]
fn recursive_collect_arb(depth: usize, fan_out: usize) {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let tree = Node::build_tree(depth, fan_out, &mut rng);

    let xap = Id::<&Node>::new()
        .map(|x| x.value)
        .filter(|x| !x.is_multiple_of(7));

    let runner = Runner::adaptive();
    let params = Params::default();

    let mut expected: Vec<_> = flatten([&tree], &|x: &&Node| &x.children)
        .into_iter()
        .flat_map(|x| xap.xap(x))
        .collect();
    expected.sort();

    let results = collect_arb(runner, params, [&tree], xap, |x| &x.children);
    let mut result = Vec::new();
    merge_arb_into_vec(results, &mut result);
    result.sort();

    assert_eq!(result, expected);
}
