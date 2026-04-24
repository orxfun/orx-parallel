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
            .num_threads(2)
            .chunk_size(0)
            .iteration_order(IterationOrder::Ordered)
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
            .num_threads(2)
            .chunk_size(0)
            .iteration_order(IterationOrder::Ordered)
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
}

// #[test]
// fn kind_into_optional() {
//     let par = get_par(42).map(Some);
//     let par = par.into_optional();
//     let result = count_opt(par);
//     assert_eq!(result, Some(42));
// }

// #[test]
// fn kind_into_optional_use() {
//     {
//         let par = get_par(42).map(Some);
//         let par = par.into_optional();

//         let u = |x: usize| x.to_string();
//         let par = par.using(u);

//         let result = count_use_opt(par);
//         assert_eq!(result, Some(42));
//     }

//     {
//         let par = get_par(42).map(Some);
//         let par = par.into_optional();

//         let u = String::from("42");
//         let par = par.using_clone(u);

//         let result = count_use_opt(par);
//         assert_eq!(result, Some(42));
//     }
// }

// #[test]
// fn kind_into_use_optional() {
//     {
//         let u = |x: usize| x.to_string();
//         let par = get_par(42);
//         let par = par.using(u);
//         let par = par.map(|_use, x| Some(x));
//         let par = par.into_optional();
//         let result = count_use_opt(par);
//         assert_eq!(result, Some(42));
//     }

//     {
//         let u = String::from("42");
//         let par = get_par(42);
//         let par = par.using_clone(u);
//         let par = par.map(|_use, x| Some(x));
//         let par = par.into_optional();
//         let result = count_use_opt(par);
//         assert_eq!(result, Some(42));
//     }
// }

// #[test]
// fn kind_into_fallible() {
//     let par = get_par(42).map(|x| Result::<_, char>::Ok(x));
//     let par = par.into_fallible();
//     let result = count_res(par);
//     assert_eq!(result, Ok(42));
// }

// #[test]
// fn kind_into_fallible_use() {
//     {
//         let par = get_par(42).map(|x| Result::<_, char>::Ok(x));
//         let par = par.into_fallible();

//         let u = |x: usize| x.to_string();
//         let par = par.using(u);

//         let result = count_use_res(par);
//         assert_eq!(result, Ok(42));
//     }

//     {
//         let par = get_par(42).map(|x| Result::<_, char>::Ok(x));
//         let par = par.into_fallible();

//         let u = String::from("42");
//         let par = par.using_clone(u);

//         let result = count_use_res(par);
//         assert_eq!(result, Ok(42));
//     }
// }

// #[test]
// fn kind_into_use_fallible() {
//     {
//         let u = |x: usize| x.to_string();
//         let par = get_par(42);
//         let par = par.using(u);

//         let par = par.map(|_use, x| Result::<_, char>::Ok(x));
//         let par = par.into_fallible();

//         let result = count_use_res(par);
//         assert_eq!(result, Ok(42));
//     }

//     {
//         let u = String::from("42");
//         let par = get_par(42);
//         let par = par.using_clone(u);

//         let par = par.map(|_use, x| Result::<_, char>::Ok(x));
//         let par = par.into_fallible();

//         let result = count_use_res(par);
//         assert_eq!(result, Ok(42));
//     }
// }

// #[test]
// fn kind_use() {
//     {
//         let u = |x: usize| x.to_string();
//         let par = get_par(42);
//         let par = par.using(u);
//         let result = count_use(par);
//         assert_eq!(result, 42);
//     }

//     {
//         let u = String::from("42");
//         let par = get_par(42);
//         let par = par.using_clone(u);
//         let result = count_use(par);
//         assert_eq!(result, 42);
//     }
// }

// // enumerate

// #[test]
// fn kind_enumerate() {
//     let par = get_par(42);
//     let par = par.enumerate();
//     let par = par.map(|(_i, x)| x);
//     let result = count(par);
//     assert_eq!(result, 42);
// }

// #[test]
// fn kind_use_enumerate() {
//     let par = get_par_use(42);
//     let par = par.enumerate();
//     let par = par.map(|u, (_i, x): (usize, String)| {
//         *u = format!("{u}!");
//         x
//     });
//     let result = count_use(par);
//     assert_eq!(result, 42);
// }
