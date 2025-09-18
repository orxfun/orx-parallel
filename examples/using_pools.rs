mod utils;

// cargo run --all-features --release --example using_pools
// to run with all options

// cargo run --all-features --release --example using_pools -- --pool-type scoped-pool
// to run only using scoped-pool

// cargo run --all-features --release --example using_pools -- --pool-type scoped-thread-pool --len 100 --num-repetitions 100000
// to run only using scoped_threadpool, with 100000 repetitions for input size of 100

// cargo run --all-features --release --example using_pools -- --pool-type sequential
// 11.02s

// cargo run --all-features --release --example using_pools -- --pool-type std
// 3.66s

// cargo run --all-features --release --example using_pools -- --pool-type pond
// 11.49s

// cargo run --all-features --release --example using_pools -- --pool-type poolite
// 12.27s

// cargo run --all-features --release --example using_pools -- --pool-type rayon-core
// 3.76s

// cargo run --all-features --release --example using_pools -- --pool-type scoped_threadpool
// 3.70s

// cargo run --all-features --release --example using_pools -- --pool-type yastl
// 11.47s

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
                                fib(x % 10),
                                fib(x % 21),
                                fib(x % 17),
                                fib(x % 33),
                                fib(x % 21),
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

        fn fib(n: usize) -> usize {
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
            let mut runner = SequentialRunner::default();
            run_with_runner(&mut runner, num_threads, num_repetitions, input)
        }

        fn run_pond(num_threads: usize, num_repetitions: usize, input: &[usize]) -> Vec<String> {
            let mut pond = PondPool::new_threads_unbounded(num_threads);
            let mut runner = RunnerWithPondPool::from(&mut pond);
            run_with_runner(&mut runner, num_threads, num_repetitions, input)
        }

        fn run_poolite(num_threads: usize, num_repetitions: usize, input: &[usize]) -> Vec<String> {
            let pond = poolite::Pool::with_builder(
                poolite::Builder::new().min(num_threads).max(num_threads),
            )
            .unwrap();
            let mut runner = RunnerWithPoolitePool::from(&pond);
            run_with_runner(&mut runner, num_threads, num_repetitions, input)
        }

        fn run_rayon_core(
            num_threads: usize,
            num_repetitions: usize,
            input: &[usize],
        ) -> Vec<String> {
            let pond = rayon_core::ThreadPoolBuilder::new()
                .num_threads(num_threads)
                .build()
                .unwrap();
            let mut runner = RunnerWithRayonPool::from(&pond);
            run_with_runner(&mut runner, num_threads, num_repetitions, input)
        }

        fn run_scoped_pool(
            num_threads: usize,
            num_repetitions: usize,
            input: &[usize],
        ) -> Vec<String> {
            let pond = scoped_pool::Pool::new(num_threads);
            let mut runner = RunnerWithScopedPool::from(&pond);
            run_with_runner(&mut runner, num_threads, num_repetitions, input)
        }

        fn run_scoped_threadpool(
            num_threads: usize,
            num_repetitions: usize,
            input: &[usize],
        ) -> Vec<String> {
            let mut pond = scoped_threadpool::Pool::new(num_threads as u32);
            let mut runner = RunnerWithScopedThreadPool::from(&mut pond);
            run_with_runner(&mut runner, num_threads, num_repetitions, input)
        }

        fn run_yastl(num_threads: usize, num_repetitions: usize, input: &[usize]) -> Vec<String> {
            let pond = YastlPool::new(num_threads);
            let mut runner = RunnerWithYastlPool::from(&pond);
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
                    fib(x % 10),
                    fib(x % 21),
                    fib(x % 17),
                    fib(x % 33),
                    fib(x % 21),
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
