#![cfg(all(feature = "wasm", target_arch = "wasm32", target_feature = "atomics"))]

use orx_parallel::{IntoParIter, Par, Pool, init_thread_pool};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
#[should_panic(expected = "not initialized")]
fn wasm_web_pool_panics_without_init() {
    let pool = Pool::wasm_web(2);

    let _: Vec<usize> = (0..32).into_par().pool(pool).collect();
}

#[wasm_bindgen_test(async)]
async fn wasm_web_pool_runs_after_init() {
    JsFuture::from(init_thread_pool(2))
        .await
        .expect("init_thread_pool should resolve");

    let pool = Pool::wasm_web(2);
    let values: Vec<usize> = (0..100).into_par().pool(pool).collect();
    let sum: usize = values.into_iter().sum();

    assert_eq!(sum, (0..100usize).sum::<usize>());
}
