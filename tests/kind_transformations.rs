/*
These tests make sure that the api of parallel iterator traits are
set up properly so that the corresponding transformation and computation
methods are available on the trait, without requiring the concrete
iterator type implementing the trait.
*/

use orx_parallel::*;
use std::string::{String, ToString};

#[test]
fn kind_inf_transform_compute() {
    fn get_par(n: usize) -> impl Par<Item = String> {
        (0..n).par().map(|x| x.to_string())
    }

    fn collect(par: impl Par<Item = String>) -> Vec<String> {
        par.num_threads(3).chunk_size(1).collect()
    }

    fn count(par: impl Par<Item = String>) -> usize {
        par.num_threads(1)
            .chunk_size(7)
            .map(|_| 1)
            .reduce(|a, b| a + b)
            .unwrap_or(0)
    }

    fn find(par: impl Par<Item = String>) -> Option<String> {
        par.filter(|x| x.len() > 2)
            .num_threads(6)
            .chunk_size(3)
            .first()
    }

    fn map(par: impl Par<Item = String>) -> impl Par<Item = String> {
        par.map(|x| format!("{x}!"))
    }

    fn filter(par: impl Par<Item = String>) -> impl Par<Item = String> {
        par.filter(|x| x.len() > 0)
    }

    fn filter_map(par: impl Par<Item = String>) -> impl Par<Item = String> {
        par.filter_map(Some)
    }

    fn flat_map(par: impl Par<Item = String>) -> impl Par<Item = String> {
        par.flat_map(|x| [x])
    }

    let par = get_par(42);
    let par = flat_map(filter_map(filter(map(par))));
    let result = collect(par);
    assert_eq!(result.len(), 42);

    let par = get_par(42);
    let par = flat_map(filter_map(filter(map(par))));
    let result = count(par);
    assert_eq!(result, 42);

    let par = get_par(42);
    let par = flat_map(filter_map(filter(map(par))));
    let result = find(par);
    assert!(result.is_some());

    fn map_to_opt(par: impl Par<Item = String>) -> impl Par<Item = Option<String>> {
        par.map(Some)
    }
    let par = map_to_opt(get_par(42));
    let par = par.into_optional();
    assert_eq!(par.first(), Some(Some(String::from("0"))));

    fn map_to_res(par: impl Par<Item = String>) -> impl Par<Item = Result<String, char>> {
        par.map(Ok)
    }
    let par = map_to_res(get_par(42));
    let par = par.into_fallible();
    assert_eq!(par.first(), Ok(Some(String::from("0"))));

    fn map_to_use(par: impl Par<Item = String>) -> impl ParUse<Use = char, Item = String> {
        par.using_clone('x')
    }
    let par = map_to_use(get_par(42));
    assert_eq!(par.first(), Some(String::from("0")));

    fn map_to_use_opt(
        par: impl Par<Item = String>,
    ) -> impl ParUse<Use = char, Item = Option<String>> {
        par.map(Some).using(|_| 'x')
    }
    let par = map_to_use_opt(get_par(42));
    let par = par.into_optional();
    assert_eq!(par.first(), Some(Some(String::from("0"))));

    fn map_to_use_res(
        par: impl Par<Item = String>,
    ) -> impl ParUse<Use = char, Item = Result<String, char>> {
        par.map(Ok).using_clone('x')
    }
    let par = map_to_use_res(get_par(42));
    let par = par.into_fallible();
    assert_eq!(par.first(), Ok(Some(String::from("0"))));
}

#[test]
fn kind_use_inf_transform_compute() {
    fn get_par(n: usize, u: char) -> impl ParUse<Use = char, Item = String> {
        (0..n).par().map(|x| x.to_string()).using_clone(u)
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
        par.filter(|_u, x| x.len() > 0)
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
}

#[test]
fn kind_opt_transform_compute() {
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
        par.using_clone('x')
    }
    let par = map_to_use(get_par(42));
    assert_eq!(par.first(), Some(Some(String::from("0"))));
}
