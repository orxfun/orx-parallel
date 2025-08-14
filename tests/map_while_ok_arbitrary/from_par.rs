use orx_parallel::*;

use crate::map_while_ok_arbitrary::utils::sort_if_ok;

#[test]
fn map_while_ok_from_par_when_ok() {
    let input = 0..1024;
    let map_res = |i: usize| match (1300..1350).contains(&i) {
        true => Err(i.to_string()),
        false => Ok(i),
    };

    let result: Result<Vec<_>, _> = input
        .into_par()
        .iteration_order(IterationOrder::Arbitrary)
        .map_while_ok(map_res)
        .collect();
    let result = sort_if_ok(result);
    let expected = Ok((0..1024).collect::<Vec<_>>());

    assert_eq!(result, expected);
}

#[test]
fn map_while_ok_from_par_when_error() {
    let input = 0..1024;
    let is_error =
        |i: &usize| (300..350).contains(i) || (400..450).contains(i) || (500..550).contains(i);
    let map_res = |i: usize| match is_error(&i) {
        true => Err(i.to_string()),
        false => Ok(i),
    };

    let result: Result<Vec<_>, _> = input
        .into_par()
        .iteration_order(IterationOrder::Arbitrary)
        .map_while_ok(map_res)
        .collect();

    let result = result.map_err(|e| {
        let number = e.parse::<usize>().unwrap();
        is_error(&number)
    });
    assert_eq!(result, Err(true));
}
