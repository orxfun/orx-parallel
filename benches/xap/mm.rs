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

pub struct Collect;
impl Exp for Collect {
    type Out = Vec<u64>;
    fn out(i: impl Iterator<Item = u64>) -> Self::Out {
        i.collect()
    }
}

fn inputs(len: usize) -> Vec<u64> {
    const SEED: u64 = 654;
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len).map(|_| rng.random_range(0..150)).collect()
}

fn f1(i: u64) -> u64 {
    2 * i + 1
}

fn f2(i: u64) -> u64 {
    (7 * i).saturating_sub(71)
}

fn iter<E: Exp>(inputs: &[u64]) -> E::Out {
    let iter = inputs.iter().copied().map(f1).map(f2);
    E::out(iter)
}

fn xap<E: Exp>(inputs: &[u64]) -> E::Out {
    let xap = Id::new().map(f1).map(f2);
    E::out(inputs.iter().copied().flat_map(|x| xap.xap(x)))
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
    let len = [1 << 10, 1 << 15, 1 << 20];
    let treatments: Vec<_> = len.into_iter().map(|n| InputVariant { n }).collect();
    let variants = vec![Method::Iter, Method::Xap];

    Bench.bench(c, "xap_mm", &treatments, &variants);
}

criterion_group!(benches, run);
criterion_main!(benches);
