use orx_parallel::*;
use std::hint::black_box;

const CPU_MIX_ROUNDS: usize = 40;
fn cpu_mix(x: u64) -> u64 {
    let mut x = black_box(x ^ 0x9E37_79B9_7F4A_7C15);
    for r in 0..CPU_MIX_ROUNDS {
        let salt = black_box((r as u64 + 1) * 0xA076_1D64_78BD_642F);
        x = black_box(x ^ salt);
        x = black_box(x.rotate_left(9).wrapping_mul(0xD6E8_FD9D_79A1_4E3B));
        x = black_box(x ^ (x >> 27));
    }
    x
}

enum Compute<Fa, Fb, Fc>
where
    Fa: FnOnce() -> u64,
    Fb: FnOnce() -> Vec<String>,
    Fc: FnOnce() -> Option<u64>,
{
    A(Fa),
    B(Fb),
    C(Fc),
}

enum Out {
    A(u64),
    B(Vec<String>),
    C(Option<u64>),
}

impl Out {
    fn a(self) -> u64 {
        match self {
            Self::A(value) => value,
            _ => unreachable!(),
        }
    }

    fn b(self) -> Vec<String> {
        match self {
            Self::B(value) => value,
            _ => unreachable!(),
        }
    }

    fn c(self) -> Option<u64> {
        match self {
            Self::C(value) => value,
            _ => unreachable!(),
        }
    }
}

impl<Fa, Fb, Fc> Compute<Fa, Fb, Fc>
where
    Fa: FnOnce() -> u64,
    Fb: FnOnce() -> Vec<String>,
    Fc: FnOnce() -> Option<u64>,
{
    fn run(self) -> Out {
        match self {
            Self::A(f) => Out::A(f()),
            Self::B(f) => Out::B(f()),
            Self::C(f) => Out::C(f()),
        }
    }
}

fn main() {
    // let seq = false;
    let n = 100_000;

    let fa = || (0..n).map(cpu_mix).sum::<u64>();
    let fb = || {
        (0..n)
            .map(cpu_mix)
            .filter(|x| !x.is_multiple_of(7))
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
    };
    let fc = || (0..n).map(cpu_mix).filter(|x| !x.is_multiple_of(13)).min();

    let computations = vec![Compute::A(fa), Compute::B(fb), Compute::C(fc)];

    let outputs: Vec<_> = computations.into_par().map(|x| x.run()).collect();
    // let outputs: Vec<_> = match seq {
    //     true => computations.into_iter().map(|x| x.run()).collect(),
    //     false => computations.into_par().map(|x| x.run()).collect(),
    // };

    let mut outputs = outputs.into_iter();
    let (a, b, c) = (
        outputs.next().unwrap().a(),
        outputs.next().unwrap().b(),
        outputs.next().unwrap().c(),
    );

    println!("a = {a}");
    println!("b.len() = {}", b.len());
    println!("c = {c:?}");
}
