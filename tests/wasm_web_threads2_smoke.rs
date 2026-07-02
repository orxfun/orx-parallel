#![cfg(all(
    feature = "wasm-web-threads2",
    target_arch = "wasm32",
    target_feature = "atomics"
))]

use orx_parallel::init_thread_pool;
use orx_parallel::{IntoParIter, Par};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test(async)]
async fn wasm_web2_init_is_idempotent_and_rejects_mismatch() {
    JsFuture::from(init_thread_pool(2))
        .await
        .expect("first init_thread_pool call should resolve");

    JsFuture::from(init_thread_pool(2))
        .await
        .expect("second init_thread_pool call with the same configuration should resolve");

    let err = JsFuture::from(init_thread_pool(3))
        .await
        .expect_err("reinitialization with a different thread count should be rejected");

    let message = err
        .as_string()
        .expect("rejection reason should be a string message");

    assert!(
        message.contains("already called with 2 threads"),
        "unexpected rejection message: {message}"
    );
}

#[wasm_bindgen_test(async)]
async fn wasm_web2_default_pool_runs_without_explicit_pool() {
    JsFuture::from(init_thread_pool(2))
        .await
        .expect("init_thread_pool should resolve for the default-path smoke test");

    let sum: usize = (0..128).into_par().sum();

    assert_eq!(sum, (0..128usize).sum::<usize>());
}
