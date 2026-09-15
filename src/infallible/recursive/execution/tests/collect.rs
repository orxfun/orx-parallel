use crate::Params;
use crate::infallible::recursive::execution::collect::collect;
use crate::infallible::recursive::execution::tests::tree::{Node, flatten};
use crate::infallible::{Xap, xap_variants::Id};
use crate::runner::default_runner;
use alloc::vec::Vec;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use test_case::test_matrix;

#[test_matrix([2, 3, 4], [2, 3, 4])]
fn recursive_collect(depth: usize, fan_out: usize) {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let tree = Node::build_tree(depth, fan_out, &mut rng);

    let xap = Id::<&Node>::new()
        .map(|x| x.value)
        .filter(|x| !x.is_multiple_of(7));

    let runner = default_runner();
    let params = Params::default();

    let mut expected: Vec<_> = flatten([&tree], &|x: &&Node| &x.children)
        .into_iter()
        .flat_map(|x| xap.xap(x))
        .collect();
    expected.sort();

    let mut result = Vec::new();
    collect(runner, params, [&tree], xap, |x| &x.children, &mut result);
    result.sort();

    assert_eq!(result, expected);
}

#[cfg(all(not(miri), feature = "long-tests"))]
#[test_matrix([4, 5, 6], [4, 5, 6])]
fn recursive_collect_determinism(depth: usize, fan_out: usize) {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let tree = Node::build_tree(depth, fan_out, &mut rng);

    let xap = Id::<&Node>::new()
        .map(|x| x.value)
        .filter(|x| !x.is_multiple_of(7));

    let mut runner = default_runner();
    let params = Params::default();

    let mut expected = Vec::new();
    collect(
        &mut runner,
        params,
        [&tree],
        xap,
        |x| &x.children,
        &mut expected,
    );

    for _ in 0..10 {
        let mut result = Vec::new();
        collect(
            &mut runner,
            params,
            [&tree],
            xap,
            |x| &x.children,
            &mut result,
        );
        assert_eq!(expected, result);
    }
}

#[cfg(all(not(miri), feature = "long-tests"))]
#[test_matrix([4, 5, 6], [4, 5, 6])]
fn recursive_collect_determinism_with_flat_map(depth: usize, fan_out: usize) {
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    let tree = Node::build_tree(depth, fan_out, &mut rng);

    let xap = Id::<&Node>::new()
        .flat_map(|x| (0..10).map(|i| x.value + i))
        .filter(|x| !x.is_multiple_of(7));

    let mut runner = default_runner();
    let params = Params::default();

    let mut expected = Vec::new();
    collect(
        &mut runner,
        params,
        [&tree],
        xap,
        |x| &x.children,
        &mut expected,
    );

    for _ in 0..10 {
        let mut result = Vec::new();
        collect(
            &mut runner,
            params,
            [&tree],
            xap,
            |x| &x.children,
            &mut result,
        );
        assert_eq!(expected, result);
    }
}

#[cfg(all(not(miri), feature = "long-tests"))]
#[test]
fn collect_large_recursion() {
    use crate::*;
    use alloc::vec;
    use core::hint::black_box;

    fn cpu_mix(rounds: usize, seed: u64) -> u64 {
        let mut x = black_box(seed ^ 0x9E37_79B9_7F4A_7C15);
        for r in 0..rounds {
            let salt = black_box(r as u64 + 1);
            x = black_box(x ^ salt);
            x = black_box(x.rotate_left(9).wrapping_mul(0xD6E8_FD9D_79A1_4E3B));
            x = black_box(x ^ (x >> 27));
        }
        x
    }

    pub struct Node {
        value: u64,
        children: Vec<Node>,
    }

    fn build_tree(depth: usize, fan_out: usize, seed: u64) -> Node {
        let children = if depth == 0 {
            vec![]
        } else {
            (0..fan_out)
                .map(|index| build_tree(depth - 1, fan_out, seed ^ index as u64))
                .collect()
        };
        Node {
            value: cpu_mix(2, seed),
            children,
        }
    }

    fn matches(node: &Node, threshold: u64) -> bool {
        let node_value = cpu_mix(1, node.value);
        node_value % 10_000 < threshold
    }

    fn collect_orx(node: &Node, threshold: u64) -> Vec<&Node> {
        par_recursive([node], |node| &node.children)
            .filter(|node| matches(node, threshold))
            .collect()
    }

    fn input(input_variant: &InputVariant) -> Node {
        build_tree(input_variant.depth, input_variant.fan_out, 42)
    }

    struct InputVariant {
        depth: usize,
        fan_out: usize,
        threshold: u64,
    }

    let input_variant = InputVariant {
        depth: 8,
        fan_out: 8,
        threshold: 5_000,
    };

    let input = input(&input_variant);
    let _result = collect_orx(&input, input_variant.threshold).len();
}
