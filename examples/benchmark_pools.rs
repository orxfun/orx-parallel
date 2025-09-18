// cargo run --all-features --release --example benchmark_pools
// to run with all options:
//
// output:
//
// Args { pool_type: All, num_threads: 16, len: 100000, num_repetitions: 1000 }
// Std => 15.912437916s
// Sequential => 46.194610858s
// Pond => 42.560279289s
// Poolite => 21.422590826s
// RayonCore => 16.227641997s
// ScopedPool => 15.958834105s
// ScopedThreadPool => 17.228307255s
// Yastl => 43.914882593s

// cargo run --all-features --release --example benchmark_pools -- --pool-type scoped-pool
// to run only using scoped-pool
//
// output:
//
// Args { pool_type: ScopedPool, num_threads: 16, len: 100000, num_repetitions: 1000 }
// ScopedPool => 16.640308686s

// cargo run --all-features --release --example benchmark_pools -- --pool-type rayon-core --len 1000 --num-repetitions 10000
// to run only using rayon-core ThreadPool, with 10000 repetitions for input size of 1000
//
// output:
//
// Args { pool_type: RayonCore, num_threads: 16, len: 1000, num_repetitions: 10000 }
// RayonCore => 6.950370104s

mod utils;

fn main() {
    #[cfg(feature = "std")]
    #[cfg(feature = "pond")]
    #[cfg(feature = "poolite")]
    #[cfg(feature = "rayon-core")]
    #[cfg(feature = "scoped-pool")]
    #[cfg(feature = "scoped_threadpool")]
    #[cfg(feature = "yastl")]
    {
        use clap::Parser;
        use orx_parallel::runner::ParallelRunner;
        use orx_parallel::*;
        use std::hint::black_box;
        use std::num::NonZeroUsize;
        use std::time::SystemTime;

        #[derive(Parser, Debug)]
        struct Args {
            /// Type of the thread pool to be used for computations.
            #[arg(long, default_value_t, value_enum)]
            pool_type: PoolType,
            /// Number of threads.
            #[arg(long, default_value_t = NonZeroUsize::new(16).unwrap())]
            num_threads: NonZeroUsize,
            /// Number of items in the input iterator.
            #[arg(long, default_value_t = 100000)]
            len: usize,
            /// Number of repetitions to measure time; total time will be reported.
            #[arg(long, default_value_t = 1000)]
            num_repetitions: usize,
        }

        #[derive(clap::ValueEnum, Clone, Copy, Default, Debug)]
        enum PoolType {
            Std,
            Sequential,
            Pond,
            Poolite,
            RayonCore,
            ScopedPool,
            ScopedThreadPool,
            Yastl,
            #[default]
            All,
        }

        impl PoolType {
            fn run_single(self, nt: usize, reps: usize, input: &[usize], expected: &[String]) {
                let now = SystemTime::now();
                let result = match self {
                    Self::Std => run_std(nt, reps, input),
                    Self::Sequential => run_sequential(nt, reps, input),
                    Self::Pond => run_pond(nt, reps, input),
                    Self::Poolite => run_poolite(nt, reps, input),
                    Self::RayonCore => run_rayon_core(nt, reps, input),
                    Self::ScopedPool => run_scoped_pool(nt, reps, input),
                    Self::ScopedThreadPool => run_scoped_threadpool(nt, reps, input),
                    Self::Yastl => run_yastl(nt, reps, input),
                    Self::All => panic!("all is handled by run_all"),
                };
                let elapsed = now.elapsed().unwrap();
                println!("{self:?} => {elapsed:?}");
                assert_eq!(expected, result);
            }

            fn run_all(nt: usize, reps: usize, input: &[usize], expected: &[String]) {
                Self::Std.run_single(nt, reps, input, expected);
                Self::Sequential.run_single(nt, reps, input, expected);
                Self::Pond.run_single(nt, reps, input, expected);
                Self::Poolite.run_single(nt, reps, input, expected);
                Self::RayonCore.run_single(nt, reps, input, expected);
                Self::ScopedPool.run_single(nt, reps, input, expected);
                Self::ScopedThreadPool.run_single(nt, reps, input, expected);
                Self::Yastl.run_single(nt, reps, input, expected);
            }

            fn run(self, nt: usize, reps: usize, input: &[usize], expected: &[String]) {
                match self {
                    Self::All => Self::run_all(nt, reps, input, expected),
                    _ => self.run_single(nt, reps, input, expected),
                }
            }
        }

        fn run_with_runner<R: ParallelRunner>(
            mut runner: R,
            num_threads: usize,
            num_repetitions: usize,
            input: &[usize],
        ) -> Vec<String> {
            let mut dummy = vec![];
            let mut result = vec![];
            for i in 0..num_repetitions {
                result = black_box(
                    input
                        .par()
                        .num_threads(num_threads)
                        .with_runner(&mut runner)
                        .flat_map(|x| {
                            [
                                *x,
                                fibonacci(x % 10),
                                fibonacci(x % 21),
                                fibonacci(x % 17),
                                fibonacci(x % 33),
                                fibonacci(x % 21),
                            ]
                        })
                        .map(|x| 3 * x)
                        .filter(|x| !(100..150).contains(x))
                        .map(|x| x.to_string())
                        .collect(),
                );
                if i < num_repetitions.min(result.len()) {
                    dummy.push(result[i].clone())
                };
            }
            for i in 0..dummy.len() {
                assert_eq!(&dummy[i], &result[i]);
            }
            result
        }

        fn fibonacci(n: usize) -> usize {
            let mut a = 0;
            let mut b = 1;
            for _ in 0..n {
                let c = a + b;
                a = b;
                b = c;
            }
            a
        }

        fn run_std(num_threads: usize, num_repetitions: usize, input: &[usize]) -> Vec<String> {
            let mut runner = DefaultRunner::default(); // StdRunner
            run_with_runner(&mut runner, num_threads, num_repetitions, input)
        }

        fn run_sequential(
            num_threads: usize,
            num_repetitions: usize,
            input: &[usize],
        ) -> Vec<String> {
            let mut runner = RunnerWithPool::from(SequentialPool);
            run_with_runner(&mut runner, num_threads, num_repetitions, input)
        }

        fn run_pond(num_threads: usize, num_repetitions: usize, input: &[usize]) -> Vec<String> {
            let mut pool = PondPool::new_threads_unbounded(num_threads);
            let mut runner = RunnerWithPool::from(&mut pool);
            run_with_runner(&mut runner, num_threads, num_repetitions, input)
        }

        fn run_poolite(num_threads: usize, num_repetitions: usize, input: &[usize]) -> Vec<String> {
            let pool = poolite::Pool::with_builder(
                poolite::Builder::new().min(num_threads).max(num_threads),
            )
            .unwrap();
            let mut runner = RunnerWithPool::from(&pool);
            run_with_runner(&mut runner, num_threads, num_repetitions, input)
        }

        fn run_rayon_core(
            num_threads: usize,
            num_repetitions: usize,
            input: &[usize],
        ) -> Vec<String> {
            let pool = rayon_core::ThreadPoolBuilder::new()
                .num_threads(num_threads)
                .build()
                .unwrap();
            let mut runner = RunnerWithPool::from(&pool);
            run_with_runner(&mut runner, num_threads, num_repetitions, input)
        }

        fn run_scoped_pool(
            num_threads: usize,
            num_repetitions: usize,
            input: &[usize],
        ) -> Vec<String> {
            let pool = scoped_pool::Pool::new(num_threads);
            let mut runner = RunnerWithPool::from(&pool);
            run_with_runner(&mut runner, num_threads, num_repetitions, input)
        }

        fn run_scoped_threadpool(
            num_threads: usize,
            num_repetitions: usize,
            input: &[usize],
        ) -> Vec<String> {
            let mut pool = scoped_threadpool::Pool::new(num_threads as u32);
            let mut runner = RunnerWithPool::from(&mut pool);
            run_with_runner(&mut runner, num_threads, num_repetitions, input)
        }

        fn run_yastl(num_threads: usize, num_repetitions: usize, input: &[usize]) -> Vec<String> {
            let pool = YastlPool::new(num_threads);
            let mut runner = RunnerWithPool::from(&pool);
            run_with_runner(&mut runner, num_threads, num_repetitions, input)
        }

        let args = Args::parse();
        println!("{args:?}");

        let input: Vec<_> = (0..args.len as usize).collect::<Vec<_>>();
        let expected: Vec<_> = input
            .iter()
            .flat_map(|x| {
                [
                    *x,
                    fibonacci(x % 10),
                    fibonacci(x % 21),
                    fibonacci(x % 17),
                    fibonacci(x % 33),
                    fibonacci(x % 21),
                ]
            })
            .map(|x| 3 * x)
            .filter(|x| !(100..150).contains(x))
            .map(|x| x.to_string())
            .collect();

        args.pool_type.run(
            args.num_threads.into(),
            args.num_repetitions,
            &input,
            &expected,
        );
    }
}
