pub enum Pool {
    Seq,

    Rayon(rayon::ThreadPool),

    #[cfg(feature = "std")]
    Basic(orx_parallel::pool::BasicPool),

    #[cfg(feature = "std")]
    Once(orx_parallel::pool::OncePool),
}

impl Pool {
    // new

    pub fn new_rayon(nt: usize) -> Self {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(nt)
            .build()
            .unwrap();
        Self::Rayon(pool)
    }

    #[cfg(feature = "std")]
    pub fn new_basic(nt: usize) -> Self {
        let pool = orx_parallel::Pool::basic(nt);
        Self::Basic(pool)
    }

    #[cfg(feature = "std")]
    pub fn new_once(nt: usize) -> Self {
        let pool = orx_parallel::Pool::once(nt);
        Self::Once(pool)
    }

    // get

    pub fn rayon(&mut self) -> &mut rayon::ThreadPool {
        match self {
            Self::Rayon(p) => p,
            _ => unreachable!(),
        }
    }

    #[cfg(feature = "std")]
    pub fn basic(&mut self) -> &mut orx_parallel::pool::BasicPool {
        match self {
            Self::Basic(p) => p,
            _ => unreachable!(),
        }
    }

    #[cfg(feature = "std")]
    pub fn once(&mut self) -> &mut orx_parallel::pool::OncePool {
        match self {
            Self::Once(p) => p,
            _ => unreachable!(),
        }
    }
}
