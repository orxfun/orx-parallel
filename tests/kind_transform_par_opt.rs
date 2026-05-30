/*
These tests make sure that the api of parallel iterator traits are
set up properly so that the corresponding transformation and computation
methods are available on the trait, without requiring the concrete
iterator type implementing the trait.
*/
use orx_parallel::*;
use std::string::{String, ToString};

#[test]
fn kind_transform_par_opt() {
    fn get_par(n: usize) -> impl ParOption<Item = String> {
        (0..n)
            .par()
            .map(|x| x.to_string())
            .map(Some)
            .into_optional()
    }

    fn collect(par: impl ParOption<Item = String>) -> Option<Vec<String>> {
        par.num_threads(3).chunk_size(1).collect()
    }

    fn count(par: impl ParOption<Item = String>) -> Option<usize> {
        par.num_threads(1)
            .chunk_size(7)
            .map(|_| 1)
            .reduce(|a, b| a + b)
            .map(|x| x.unwrap_or(0))
    }

    fn find(par: impl ParOption<Item = String>) -> Option<Option<String>> {
        par.filter(|x| x.len() > 2)
            .num_threads(6)
            .chunk_size(3)
            .first()
    }

    fn map(par: impl ParOption<Item = String>) -> impl ParOption<Item = String> {
        par.map(|x| format!("{x}!"))
    }

    fn filter(par: impl ParOption<Item = String>) -> impl ParOption<Item = String> {
        par.filter(|x| x.len() > 0)
    }

    fn filter_map(par: impl ParOption<Item = String>) -> impl ParOption<Item = String> {
        par.filter_map(Some)
    }

    fn flat_map(par: impl ParOption<Item = String>) -> impl ParOption<Item = String> {
        par.flat_map(|x| [x])
    }

    let par = get_par(42);
    let par = flat_map(filter_map(filter(map(par))));
    let result = collect(par).unwrap();
    assert_eq!(result.len(), 42);

    let par = get_par(42);
    let par = flat_map(filter_map(filter(map(par))));
    let result = count(par).unwrap();
    assert_eq!(result, 42);

    let par = get_par(42);
    let par = flat_map(filter_map(filter(map(par))));
    let result = find(par).unwrap();
    assert!(result.is_some());

    fn map_to_use(
        par: impl ParOption<Item = String>,
    ) -> impl ParUseOption<Use = char, Item = String> {
        par.use_new(|_| 'x')
    }
    let par = map_to_use(get_par(42));
    assert_eq!(par.first(), Some(Some(String::from("0"))));

    // copied & cloned
    fn get_ref_par<T: Sync>(values: &[T]) -> impl ParOption<Item = &T> {
        values.par().map(Some).into_optional()
    }
    let vals: Vec<_> = (0..42).collect();
    let par = get_ref_par(&vals).copied();
    assert_eq!(par.first().unwrap(), Some(0));
    let vals: Vec<_> = (0..42).map(|x| x.to_string()).collect();
    let par = get_ref_par(&vals).cloned();
    let par = flat_map(filter_map(filter(map(par))));
    let result = find(par);
    assert!(result.is_some());
}
