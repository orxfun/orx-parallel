use orx_parallel::*;

use crate::map_while_ok_arbitrary::utils::sort_if_ok;

#[test]
fn map_while_ok_from_xap_flat_map_when_ok() {
    let input = 0..1024;
    let flat_map = |i: usize| [i, 1000 + i, 2000 + i];
    let map_res = |i: usize| match (11300..11350).contains(&i) {
        true => Err(i.to_string()),
        false => Ok(i),
    };

    let result: Result<Vec<_>, _> = input
        .into_par()
        .iteration_order(IterationOrder::Arbitrary)
        .flat_map(flat_map)
        .map_while_ok(map_res)
        .collect();
    let expected = Ok((0..1024).flat_map(flat_map).collect::<Vec<_>>());

    let result = sort_if_ok(result);
    let expected = sort_if_ok(expected);

    assert_eq!(result, expected);
}

#[test]
fn map_while_ok_from_xap_flat_map_when_error() {
    let input = 0..1024;
    let flat_map = |i: usize| [i, 1000 + i, 2000 + i];
    let is_error =
        |i: &usize| (300..350).contains(i) || (400..450).contains(i) || (500..550).contains(i);
    let map_res = |i: usize| match is_error(&i) {
        true => Err(i.to_string()),
        false => Ok(i),
    };

    let result: Result<Vec<_>, _> = input
        .into_par()
        .iteration_order(IterationOrder::Arbitrary)
        .flat_map(flat_map)
        .map_while_ok(map_res)
        .collect();

    let result = result.map_err(|e| {
        let number = e.parse::<usize>().unwrap();
        is_error(&number)
    });
    assert_eq!(result, Err(true));
}

#[test]
fn map_while_ok_from_xap_flat_map_whilst_when_ok() {
    let input = 0..1024;
    let flat_map = |i: usize| [i, 1000 + i, 2000 + i];
    let map_res = |i: usize| match (1300..1350).contains(&i) {
        true => Err(i.to_string()),
        false => Ok(i),
    };

    let result: Result<Vec<_>, _> = input
        .clone()
        .into_par()
        .iteration_order(IterationOrder::Arbitrary)
        .flat_map(flat_map)
        .take_while(|i| i < &777)
        .map_while_ok(map_res)
        .collect();
    let expected = input
        .flat_map(flat_map)
        .take_while(|i| i < &777)
        .map(map_res)
        .collect();

    let result = sort_if_ok(result);
    let expected = sort_if_ok(expected);

    assert_eq!(result, expected);
}

#[test]
fn map_while_ok_from_xap_flat_map_whilst_when_error() {
    let input = 0..1024;
    let flat_map = |i: usize| [i, 1000 + i, 2000 + i];
    let is_error =
        |i: &usize| (300..350).contains(i) || (400..450).contains(i) || (500..550).contains(i);
    let map_res = |i: usize| match is_error(&i) {
        true => Err(i.to_string()),
        false => Ok(i),
    };

    let result: Result<Vec<_>, _> = input
        .into_par()
        .iteration_order(IterationOrder::Arbitrary)
        .flat_map(flat_map)
        .take_while(|i| i < &2777)
        .map_while_ok(map_res)
        .collect();

    let result = result.map_err(|e| {
        let number = e.parse::<usize>().unwrap();
        number < 2777 && is_error(&number)
    });
    assert_eq!(result, Err(true));
}

#[test]
fn map_while_ok_from_xap_flat_map_whilst_when_error_out_of_reach() {
    let input = 0..1024;
    let flat_map = |i: usize| [i, 1000 + i, 2000 + i];
    let is_error = |i: &usize| (800..850).contains(i);
    let map_res = |i: usize| match is_error(&i) {
        true => Err(i.to_string()),
        false => Ok(i),
    };

    let result: Result<Vec<_>, _> = input
        .into_par()
        .iteration_order(IterationOrder::Arbitrary)
        .flat_map(flat_map)
        .take_while(|i| i < &777)
        .map_while_ok(map_res)
        .collect();

    let expected = Ok((0..1024)
        .flat_map(flat_map)
        .take_while(|i| i < &777)
        .collect::<Vec<_>>());

    let result = sort_if_ok(result);
    let expected = sort_if_ok(expected);

    assert_eq!(result, expected);
}
