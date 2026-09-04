use orx_parallel::*;

#[test]
fn infallible_exact_size() {
    let values = (0..10).into_par();
    assert_eq!(values.len(), 10);
    assert!(!values.is_empty());

    let values = (0..10).into_par().map(|x| x + 1);
    assert_eq!(values.len(), 10);
}

#[test]
fn infallible_use_exact_size() {
    let values = (0..10).into_par().use_new(|_| ());
    assert_eq!(values.len(), 10);
    assert!(!values.is_empty());

    let values = (0..10).into_par().use_new(|_| ()).map(|_, x| x + 1);
    assert_eq!(values.len(), 10);
}

#[test]
fn option_exact_size() {
    let values = (0..10).into_par().map(Some).into_optional();
    assert_eq!(values.len(), 10);
    assert!(!values.is_empty());

    let values = (0..10).into_par().map(Some).into_optional().map(|x| x + 1);
    assert_eq!(values.len(), 10);
}

#[test]
fn option_use_exact_size() {
    let values = (0..10).into_par().map(Some).into_optional().use_new(|_| ());
    assert_eq!(values.len(), 10);
    assert!(!values.is_empty());

    let values = (0..10)
        .into_par()
        .map(Some)
        .into_optional()
        .use_new(|_| ())
        .map(|_, x| x + 1);
    assert_eq!(values.len(), 10);
}

#[test]
fn result_exact_size() {
    let values = (0..10).into_par().map(Ok::<_, ()>).into_fallible();
    assert_eq!(values.len(), 10);
    assert!(!values.is_empty());

    let values = (0..10)
        .into_par()
        .map(Ok::<_, ()>)
        .into_fallible()
        .map(|x| x + 1);
    assert_eq!(values.len(), 10);
}

#[test]
fn result_use_exact_size() {
    let values = (0..10)
        .into_par()
        .map(Ok::<_, ()>)
        .into_fallible()
        .use_new(|_| ());
    assert_eq!(values.len(), 10);
    assert!(!values.is_empty());

    let values = (0..10)
        .into_par()
        .map(Ok::<_, ()>)
        .into_fallible()
        .use_new(|_| ())
        .map(|_, x| x + 1);
    assert_eq!(values.len(), 10);
}
