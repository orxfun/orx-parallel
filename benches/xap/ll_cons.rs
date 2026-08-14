use criterion::{Criterion, criterion_group, criterion_main};
use orx_criterion::{Experiment, Factors};
use orx_parallel::infallible::{Xap, xap_variants::Id};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

type Output = Sum;

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

pub struct SumByLoop;
impl Exp for SumByLoop {
    type Out = u64;
    fn out(i: impl Iterator<Item = u64>) -> Self::Out {
        let mut v = 0;
        for x in i {
            v += x;
        }
        v
    }
}

pub struct Reduce;
impl Exp for Reduce {
    type Out = Option<u64>;
    fn out(i: impl Iterator<Item = u64>) -> Self::Out {
        i.reduce(|x, y| 2 * x + y + 7)
    }
}

pub struct Collect;
impl Exp for Collect {
    type Out = Vec<u64>;
    fn out(i: impl Iterator<Item = u64>) -> Self::Out {
        i.collect()
    }
}

pub struct CollectByLoop;
impl Exp for CollectByLoop {
    type Out = Vec<u64>;
    fn out(i: impl Iterator<Item = u64>) -> Self::Out {
        let mut v = Vec::new();
        for x in i {
            v.push(x);
        }
        v
    }
}

fn inputs(len: usize) -> Vec<u64> {
    const SEED: u64 = 654;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len).map(|_| rng.random_range(0..150)).collect()
}

fn f1(i: u64) -> [u64; 7] {
    [i + 1, i * 2, i + 5, i + 4, i, i.saturating_sub(3), 7 * i]
}

fn f2(i: u64) -> [u64; 3] {
    [i * 2 + 1, i, i.saturating_sub(7)]
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied().flat_map(f1).flat_map(f2);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().flat_map(f1).flat_map(f2);
    let inputs = inputs.iter().copied();
    let iter = inputs.flat_map(|x| xap.xap(x));
    E::out(iter)
}

#[derive(Clone, Copy)]
struct InputVariant {
    n: usize,
}

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["n"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![self.n.to_string()]
    }
}

enum Method {
    Iter,
    Xap,
}

impl Factors for Method {
    fn factor_names() -> Vec<&'static str> {
        vec!["method"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![match self {
            Self::Iter => "iter".to_string(),
            Self::Xap => "xap".to_string(),
        }]
    }
}

struct Bench;

impl Experiment for Bench {
    type InputFactors = InputVariant;

    type AlgFactors = Method;

    type Input = Vec<u64>;

    type Output = <Output as Exp>::Out;

    type GroupArtifact = ();

    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input {
        inputs(input_variant.n)
    }

    fn group_artifact(
        &mut self,
        _: &Self::InputFactors,
        _: &Self::AlgFactors,
        _: &Self::Input,
    ) -> Self::GroupArtifact {
    }

    fn execute(
        &mut self,
        _: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
        _: &mut Self::GroupArtifact,
    ) -> Self::Output {
        match alg_variant {
            Method::Iter => iter::<Output>(input),
            Method::Xap => xap::<Output>(input),
        }
    }

    fn expected_output(&self, _: &Self::InputFactors, input: &Self::Input) -> Option<Self::Output> {
        Some(iter::<Output>(input))
    }
}

fn run(c: &mut Criterion) {
    let len = [1 << 10, 1 << 15];
    let treatments: Vec<_> = len.into_iter().map(|n| InputVariant { n }).collect();
    let variants = vec![Method::Iter, Method::Xap];

    Bench.bench(c, "xap_ll_cons", &treatments, &variants);
}

criterion_group!(benches, run);
criterion_main!(benches);
