use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use orx_parallel::prelude::*;
use orx_priority_queue::{BinaryHeap, PriorityQueue};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

const SEED: u64 = 74135;
type Distances = Vec<u64>;
type Weights = Vec<Distances>;
type WeightsSplit = SplitVec<Distances>;
type WeightsSplitRec = SplitVec<Distances, Recursive>;

fn weights(len: usize) -> Weights {
    let mut rng = ChaCha8Rng::seed_from_u64(SEED);
    (0..len)
        .map(|_| (0..len).map(|_| rng.gen_range(0..134)).collect())
        .collect()
}

fn single_source_all_destinations(weights: &Weights, s: usize) -> Distances {
    let len = weights.len();
    let mut queue = BinaryHeap::with_capacity(len);
    let mut distances = vec![u64::MAX; len];

    distances[s] = 0;
    queue.push(s, 0);

    while let Some((position, cost)) = queue.pop() {
        if cost > distances[position] {
            continue;
        }

        for head in (0..len).filter(|i| *i != position) {
            let weight = weights[position][head];

            if distances[head] > cost + weight {
                distances[head] = cost + weight;
                queue.push(head, cost + weight);
            }
        }
    }

    distances
}

fn seq(weights: &Weights) -> Weights {
    (0..weights.len())
        .map(|s| single_source_all_destinations(weights, s))
        .collect()
}

fn rayon(weights: &Weights) -> Weights {
    (0..weights.len())
        .into_par_iter()
        .map(|s| single_source_all_destinations(weights, s))
        .collect()
}

fn orx_parallel_vec(weights: &Weights) -> Weights {
    (0..weights.len())
        .par()
        .map(|s| single_source_all_destinations(weights, s))
        .collect_vec()
}

fn orx_parallel_split_vec(weights: &Weights) -> WeightsSplit {
    (0..weights.len())
        .par()
        .map(|s| single_source_all_destinations(weights, s))
        .collect()
}

fn orx_parallel_split_rec(weights: &Weights) -> WeightsSplitRec {
    (0..weights.len())
        .par()
        .map(|s| single_source_all_destinations(weights, s))
        .collect_x()
}

fn orx_parallel(weights: &Weights, num_threads: usize, chunk_size: usize) -> WeightsSplit {
    let len = weights.len();
    (0..len)
        .par()
        .chunk_size(chunk_size)
        .num_threads(num_threads)
        .map(|s| single_source_all_destinations(weights, s))
        .collect()
}

fn all_pairs_dijkstra(c: &mut Criterion) {
    let treatments = [64, 512, 1024];
    let params: Vec<(usize, usize)> = vec![(0, 1), (16, 1)];

    let mut group = c.benchmark_group("all_pairs_dijkstra");

    for n in &treatments {
        let weights = weights(*n);
        let expected = seq(black_box(&weights))[3][2];

        group.bench_with_input(BenchmarkId::new("seq", n), n, |b, _| {
            b.iter(|| {
                let result = seq(black_box(&weights));
                assert_eq!(result[3][2], expected);
            })
        });

        group.bench_with_input(BenchmarkId::new("rayon", n), n, |b, _| {
            b.iter(|| {
                let result = rayon(black_box(&weights));
                assert_eq!(result[3][2], expected);
            })
        });

        group.bench_with_input(BenchmarkId::new("orx-parallel-split-vec", n), n, |b, _| {
            b.iter(|| {
                let result = orx_parallel_split_vec(black_box(&weights));
                assert_eq!(result[3][2], expected);
            })
        });

        group.bench_with_input(BenchmarkId::new("orx-parallel-vec", n), n, |b, _| {
            b.iter(|| {
                let result = orx_parallel_vec(black_box(&weights));
                assert_eq!(result[3][2], expected);
            })
        });

        group.bench_with_input(BenchmarkId::new("orx-parallel-rec", n), n, |b, _| {
            b.iter(|| {
                let _result = orx_parallel_split_rec(black_box(&weights));
            })
        });

        for (t, c) in params.iter().copied() {
            let name = format!("orx-parallel-t{}-c{}", t, c);
            group.bench_with_input(BenchmarkId::new(name, n), n, |b, _| {
                b.iter(|| {
                    let result = orx_parallel(black_box(&weights), t, c);
                    assert_eq!(result[3][2], expected);
                })
            });
        }
    }

    group.finish();
}

criterion_group!(benches, all_pairs_dijkstra);
criterion_main!(benches);
