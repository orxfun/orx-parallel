use crate::infallible::{Xap, xap_variants::Id};
use crate::recursive::infallible::execution::next_any::next_any;
use crate::recursive::infallible::execution::tests::tree::{Node, RANGE};
use crate::{Params, Runner};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use test_case::test_matrix;

#[test_matrix([2, 3, 4], [2, 3, 4], [true, false])]
fn recursive_next_any(depth: usize, fan_out: usize, find: bool) {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let tree = Node::build_tree(depth, fan_out, &mut rng);

    let filter = move |x: &u64| match find {
        true => *x < RANGE.end / 2,
        false => *x > RANGE.end + 1,
    };

    let xap = Id::<&Node>::new().map(|x| x.value).filter(filter);
    let runner = Runner::adaptive();
    let params = Params::default();

    let result = next_any(runner, params, [&tree], xap, |x| &x.children);

    match find {
        true => assert!(result.map(|x| filter(&x)).unwrap_or(true)),
        false => assert!(result.is_none()),
    }
}
