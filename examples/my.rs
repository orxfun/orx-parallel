use orx_parallel::xap::{
    Id, Xap, XapCopied, count::iter::FlatMapIterMany, fun::flat_map::FnFlatMap,
};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

type Output = Collect;

trait Exp {
    type Out;
    fn out(i: impl Iterator<Item = u64>) -> Self::Out;
}

pub struct Sum;
impl Exp for Sum {
    type Out = u64;
    fn out(i: impl Iterator<Item = u64>) -> Self::Out {
        i.sum()
    }
}

pub struct Collect;
impl Exp for Collect {
    type Out = Vec<u64>;
    fn out(i: impl Iterator<Item = u64>) -> Self::Out {
        i.collect()
    }
}

fn inputs(len: usize) -> Vec<u64> {
    return vec![1, 2, 3];
    const SEED: u64 = 654;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len).map(|_| rng.random_range(0..150)).collect()
}

fn f1(i: u64) -> Vec<u64> {
    // vec![i + 1, i * 2, i + 5, i + 4, i, i.saturating_sub(3), 7 * i]
    (0..3).map(|x| x + i).collect()
}

fn f2(i: u64) -> Vec<u64> {
    // vec![i * 2 + 1, i, i.saturating_sub(7)]
    (0..2).map(|x| 10 * (x + 1) + i).collect()
}

fn f3(i: u64) -> Vec<u64> {
    // vec![i, 100 * i]
    vec![i / 3, i + 7, i.saturating_sub(4), i / 4, i]
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs
        .iter()
        .copied()
        .flat_map(f1)
        .flat_map(f2)
        // .flat_map(f3)
        // abc
        ;
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    // let it0 = inputs.iter().copied().flat_map(|x| [x]);
    // let it1 = FlatMapIterMany::new(it0, FnFlatMap::new(f1));
    // let it2 = FlatMapIterMany::new(it1, FnFlatMap::new(f2));
    // return E::out(it2);
    // let it3 = FlatMapIterMany::new(it2, FnFlatMap::new(f3));
    // return E::out(it3);
    let xap = Id::new().flat_map(f1).flat_map(f2);
    // let xap = Id::new().copied().flat_map(f1).flat_map(f2);
    // let xap = Id::new().flat_map(f1).flat_map(f2).flat_map(f3);
    E::out(inputs.iter().copied().flat_map(|x| {
        let xyz = xap.xap(x);
        let zyx: Vec<_> = xyz.collect();
        xap.xap(x)
    }))
}

fn main() {
    let len = [1 << 10, 1 << 15, 1 << 17];
    let len = [3];

    for n in len {
        let input = inputs(n);
        let expected = iter::<Output>(&input);

        let x = xap::<Output>(&input);

        println!("{expected:?}");
        dbg!(expected.len());

        assert_eq!(x, expected);
    }
}
