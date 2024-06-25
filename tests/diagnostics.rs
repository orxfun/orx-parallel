// #![allow(unused_variables, dead_code)]

// use orx_concurrent_iter::IntoConcurrentIter;
// use orx_parallel::{ChunkSize, IntoPar, Reduce};
// use rand::prelude::*;
// use rand_chacha::ChaCha8Rng;
// use std::{hint::black_box, num::NonZeroUsize};

// #[test]
// fn diag_reduce_sum() {
//     let len = 262_144 * 16;
//     let num_threads = 4;
//     let chunk_size = NonZeroUsize::new(1024).unwrap();

//     fn red(a: usize, b: usize) -> usize {
//         a + b
//     }
//     fn seq(inputs: &[usize]) -> Option<usize> {
//         inputs.iter().copied().reduce(red)
//     }

//     let mut rng = ChaCha8Rng::seed_from_u64(874845);
//     let vec: Vec<usize> = (0..len).map(|_| rng.gen_range(0..15791)).collect();
//     let inputs = vec.as_slice();

//     let expected = seq(inputs);

//     let result = inputs
//         .into_con_iter()
//         .cloned()
//         .into_par()
//         .num_threads(num_threads)
//         .chunk_size(ChunkSize::Exact(chunk_size))
//         .reduce(red);

//     assert_eq!(result, expected);
// }

// #[test]
// fn diag_reduce_black_box_sum() {
//     let len = 262_144 * 16;
//     let num_threads = 4;
//     let chunk_size = 1024;

//     fn red(a: usize, b: usize) -> usize {
//         black_box(black_box(a) + black_box(b))
//     }
//     fn seq(inputs: &[usize]) -> Option<usize> {
//         inputs.iter().copied().reduce(red)
//     }

//     let mut rng = ChaCha8Rng::seed_from_u64(874845);
//     let vec: Vec<usize> = (0..len).map(|_| rng.gen_range(0..15791)).collect();
//     let inputs = vec.as_slice();

//     let expected = seq(inputs);

//     let result = inputs
//         .into_con_iter()
//         .cloned()
//         .into_par()
//         // .num_threads(num_threads)
//         // .chunk_size(ChunkSize::Exact(chunk_size))
//         .reduce(red);

//     assert_eq!(result, expected);
// }

// #[test]
// fn diag_nested_reduction() {
//     let num_threads = 4;
//     let chunk_size = 1024;

//     const LEN_1: usize = 512;
//     const LEN_2: usize = 512;
//     const LEN_3: usize = 16;

//     fn seq() -> usize {
//         (0..LEN_1)
//             .map(|x| {
//                 (0..LEN_2)
//                     .map(|y| (0..LEN_3).map(|z| black_box(x * y * z)).sum::<usize>())
//                     .sum::<usize>()
//             })
//             .sum::<usize>()
//     }

//     fn orx_parallel_1(num_threads: usize, chunk_size: usize) -> usize {
//         (0..LEN_1)
//             .into_par()
//             // .num_threads(num_threads)
//             // .chunk_size(chunk_size)
//             .map(|x| {
//                 (0..LEN_2)
//                     .map(|y| (0..LEN_3).map(|z| black_box(x * y * z)).sum::<usize>())
//                     .sum::<usize>()
//             })
//             .sum()
//     }

//     fn orx_parallel_12(num_threads: usize, chunk_size: usize) -> usize {
//         (0..LEN_1)
//             .into_par()
//             // .num_threads(num_threads)
//             // .chunk_size(chunk_size)
//             .map(|x| {
//                 (0..LEN_2)
//                     .into_par()
//                     // .num_threads(num_threads)
//                     // .chunk_size(chunk_size)
//                     .map(|y| (0..LEN_3).map(|z| black_box(x * y * z)).sum())
//                     .sum()
//             })
//             .sum()
//     }

//     let expected = seq();

//     // let result = orx_parallel_1(num_threads, chunk_size);
//     let result = orx_parallel_12(num_threads, chunk_size);

//     assert_eq!(result, expected);
// }

// #[test]
// fn diag_fmap_large_output() {
//     use criterion::black_box;
//     use orx_parallel::*;
//     use rand::prelude::*;
//     use rand_chacha::ChaCha8Rng;
//     use std::num::NonZeroUsize;

//     const SEED: u64 = 3134;
//     const FIB_UPPER_BOUND: u32 = 30;

//     #[derive(PartialEq, Eq, Debug)]
//     struct LargeOutput {
//         idx: usize,
//         name: String,
//         numbers: [i64; 64],
//         vectors: Vec<Vec<i64>>,
//     }

//     // todo! why do we need Default?
//     impl Default for LargeOutput {
//         fn default() -> Self {
//             Self {
//                 idx: 0,
//                 name: Default::default(),
//                 numbers: [0; 64],
//                 vectors: Default::default(),
//             }
//         }
//     }

//     fn to_large_output(idx: u32) -> LargeOutput {
//         let idx = idx as usize;
//         let prefix = match idx % 7 {
//             0 => "zero-",
//             3 => "three-",
//             _ => "sth-",
//         };
//         let fib = fibonacci(idx as u32 % 50);
//         let name = format!("{}-fib-{}", prefix, fib);

//         let mut numbers = [0i64; 64];
//         for (i, x) in numbers.iter_mut().enumerate() {
//             *x = match (idx * 7 + i) % 3 {
//                 0 => idx as i64 + i as i64,
//                 1 => idx as i64 - i as i64,
//                 _ => fibonacci((idx % 30) as u32) as i64,
//             };
//         }

//         let mut vectors = vec![];
//         for i in 0..8 {
//             let mut vec = vec![];
//             for j in 0..idx % 1024 {
//                 vec.push(idx as i64 - i as i64 + j as i64);
//             }
//             vectors.push(vec);
//         }

//         LargeOutput {
//             idx,
//             name,
//             numbers,
//             vectors,
//         }
//     }

//     impl PartialOrd for LargeOutput {
//         fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
//             self.idx.partial_cmp(&other.idx)
//         }
//     }
//     impl Ord for LargeOutput {
//         fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//             self.idx.cmp(&other.idx)
//         }
//     }

//     fn fibonacci(n: u32) -> usize {
//         let mut a = 0;
//         let mut b = 1;
//         for _ in 0..n {
//             let c = a + b;
//             a = b;
//             b = c;
//         }
//         a as usize
//     }

//     fn inputs(len: usize) -> Vec<u32> {
//         let mut rng = ChaCha8Rng::seed_from_u64(SEED);
//         (0..len)
//             .map(|_| rng.gen_range(0..FIB_UPPER_BOUND))
//             .collect()
//     }

//     fn map(a: &u32) -> Vec<LargeOutput> {
//         [*a, a + 1, a + 2, a + 13]
//             .map(to_large_output)
//             .into_iter()
//             .collect()
//     }

//     fn validate(mut first: Vec<LargeOutput>, second: &[LargeOutput]) {
//         first.sort();
//         assert_eq!(first, second);
//     }

//     fn seq(inputs: &[u32]) -> Vec<LargeOutput> {
//         inputs.iter().flat_map(map).collect()
//     }

//     fn orx_parallel(inputs: &[u32], num_threads: usize, chunk_size: usize) -> Vec<LargeOutput> {
//         inputs
//             .into_par()
//             .chunk_size(ChunkSize::Exact(NonZeroUsize::new(chunk_size).unwrap()))
//             .num_threads(num_threads)
//             .flat_map(map)
//             .collect_vec()
//     }

//     let len = 65_536 * 4;
//     let num_threads = 16;
//     let chunk_size = len / num_threads;

//     let n = &len;
//     let input = inputs(*n);
//     let mut expected = seq(&input);
//     expected.sort();

//     let result = orx_parallel(black_box(&input), num_threads, chunk_size);
//     validate(result, &expected);
// }
