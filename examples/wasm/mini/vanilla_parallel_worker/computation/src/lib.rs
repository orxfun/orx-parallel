use orx_parallel::*;
use std::hint::black_box;

fn cpu_mix(x: u64) -> u64 {
    const CPU_MIX_ROUNDS: usize = 500;
    let mut x = black_box(x ^ 0x9E37_79B9_7F4A_7C15);
    for r in 0..CPU_MIX_ROUNDS {
        let salt = black_box((r as u64 + 1).wrapping_mul(0xA076_1D64_78BD_642F));
        x = black_box(x ^ salt);
        x = black_box(x.rotate_left(9).wrapping_mul(0xD6E8_FD9D_79A1_4E3B));
        x = black_box(x ^ (x >> 27));
    }
    x
}

pub fn compute(input: usize, num_threads: usize) -> u64 {
    (0..input)
        .par()
        .num_threads(num_threads)
        .filter(|x| !x.is_multiple_of(42))
        .map(|x| cpu_mix(x as u64))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = compute(3, 8);
        assert_eq!(result, 787191910200961162);
    }
}
