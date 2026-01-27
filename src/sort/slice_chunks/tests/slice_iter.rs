#[test]
fn xyz() {
    let p = core::ptr::null::<alloc::string::String>();
    let q = core::ptr::null::<alloc::string::String>();

    assert_eq!(p, q);
}
