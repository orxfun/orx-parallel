use crate::*;
use std::sync::atomic::{AtomicUsize, Ordering};

#[test]
fn tasks_macro() {
    let counter = AtomicUsize::new(0);

    let t0 = tasks![];
    Pool::global().run_all(t0);
    assert_eq!(counter.load(Ordering::Relaxed), 0);

    let t1 = tasks![|| {
        counter.fetch_add(1, Ordering::Relaxed);
    }];
    Pool::global().run_all(t1);
    assert_eq!(counter.load(Ordering::Relaxed), 1);

    let t3 = tasks![
        || {
            counter.fetch_add(10, Ordering::Relaxed);
        },
        || {
            counter.fetch_add(100, Ordering::Relaxed);
        },
        || {
            counter.fetch_add(1000, Ordering::Relaxed);
        },
    ];
    Pool::global().run_all(t3);
    assert_eq!(counter.load(Ordering::Relaxed), 1111);
}
