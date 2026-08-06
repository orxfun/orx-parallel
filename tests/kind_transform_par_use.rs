/*
These tests make sure that the api of parallel iterator traits are
set up properly so that the corresponding transformation and computation
methods are available on the trait, without requiring the concrete
iterator type implementing the trait.
*/
use orx_parallel::*;
use std::string::{String, ToString};

#[test]
fn kind_transform_par_use() {
    fn get_par(n: usize, u: char) -> impl EnumerateParUse<Use = char, Item = String> {
        (0..n).par().map(|x| x.to_string()).use_new(move |_| u)
    }

    fn collect(par: impl ParUse<Use = char, Item = String>) -> Vec<String> {
        par.num_threads(3).chunk_size(1).collect()
    }

    fn count(par: impl ParUse<Use = char, Item = String>) -> usize {
        par.num_threads(1)
            .chunk_size(7)
            .map(|_u, _| 1)
            .reduce(|_u, a, b| a + b)
            .unwrap_or(0)
    }

    fn find(par: impl ParUse<Use = char, Item = String>) -> Option<String> {
        par.filter(|_u, x| x.len() > 2)
            .num_threads(6)
            .chunk_size(3)
            .first()
    }

    fn map(par: impl ParUse<Use = char, Item = String>) -> impl ParUse<Use = char, Item = String> {
        par.map(|_u, x| format!("{x}!"))
    }

    fn filter(
        par: impl ParUse<Use = char, Item = String>,
    ) -> impl ParUse<Use = char, Item = String> {
        par.filter(|_u, x| !x.is_empty())
    }

    fn filter_map(
        par: impl ParUse<Use = char, Item = String>,
    ) -> impl ParUse<Use = char, Item = String> {
        par.filter_map(|_u, x| Some(x))
    }

    fn flat_map(
        par: impl ParUse<Use = char, Item = String>,
    ) -> impl ParUse<Use = char, Item = String> {
        par.flat_map(|_, x| [x])
    }

    let par = get_par(42, 'x');
    let par = flat_map(filter_map(filter(map(par))));
    let result = collect(par);
    assert_eq!(result.len(), 42);

    let par = get_par(42, 'x');
    let par = flat_map(filter_map(filter(map(par))));
    let result = count(par);
    assert_eq!(result, 42);

    let par = get_par(42, 'x');
    let par = flat_map(filter_map(filter(map(par))));
    let result = find(par);
    assert!(result.is_some());

    fn map_to_opt(
        par: impl ParUse<Use = char, Item = String>,
    ) -> impl ParUse<Use = char, Item = Option<String>> {
        par.map(|_u, x| Some(x))
    }
    let par = map_to_opt(get_par(42, 'x'));
    let par = par.into_optional();
    assert_eq!(par.first(), Some(Some(String::from("0"))));

    fn map_to_res(
        par: impl ParUse<Use = char, Item = String>,
    ) -> impl ParUse<Use = char, Item = Result<String, char>> {
        par.map(|_u, x| Ok(x))
    }
    let par = map_to_res(get_par(42, 'x'));
    let par = par.into_fallible();
    assert_eq!(par.first(), Ok(Some(String::from("0"))));

    // copied & cloned
    fn get_ref_par<T: Sync>(values: &[T]) -> impl ParUse<Use = char, Item = &T> {
        values.par().use_new(|_| 'x')
    }
    let vals: Vec<_> = (0..42).collect();
    let par = get_ref_par(&vals).copied();
    assert_eq!(par.first().unwrap(), 0);
    let vals: Vec<_> = (0..42).map(|x| x.to_string()).collect();
    let par = get_ref_par(&vals).cloned();
    let par = flat_map(filter_map(filter(map(par))));
    let result = find(par);
    assert!(result.is_some());

    // enumerate
    let par = get_par(42, 'x').enumerate();
    assert_eq!(par.first().unwrap(), (0, "0".to_string()));
}
