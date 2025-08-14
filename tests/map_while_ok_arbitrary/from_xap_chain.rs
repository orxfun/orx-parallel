use crate::map_while_ok_arbitrary::utils::sort_if_ok;
use orx_parallel::*;

#[test]
fn map_while_ok_from_xap_chain_when_ok() {
    let input = 0..1024;
    let flat_map = |i: usize| [i, 1000 + i, 2000 + i];
    let filter = |i: &usize| i < &2000;
    let map = |i: usize| 2 * i;
    let filter_map = |i: usize| (i % 2 == 0).then_some(i);
    let map2 = |i: usize| i / 2;
    let map_res = |i: usize| match (11300..11350).contains(&i) {
        true => Err(i.to_string()),
        false => Ok(i),
    };

    let result: Result<Vec<_>, _> = input
        .into_par()
        .iteration_order(IterationOrder::Arbitrary)
        .flat_map(flat_map)
        .filter(filter)
        .map(map)
        .filter_map(filter_map)
        .map(map2)
        .map_while_ok(map_res)
        .collect();
    let expected = Ok((0..1024)
        .flat_map(flat_map)
        .filter(filter)
        .map(map)
        .filter_map(filter_map)
        .map(map2)
        .collect::<Vec<_>>());

    let result = sort_if_ok(result);
    let expected = sort_if_ok(expected);

    assert_eq!(result, expected);
}

#[test]
fn map_while_ok_from_xap_chain_when_error() {
    let input = 0..1024;
    let flat_map = |i: usize| [i, 1000 + i, 2000 + i];
    let filter = |i: &usize| i < &2000;
    let map = |i: usize| 2 * i;
    let filter_map = |i: usize| (i % 2 == 0).then_some(i);
    let map2 = |i: usize| i / 2;
    let is_error =
        |i: &usize| (300..350).contains(i) || (400..450).contains(i) || (500..550).contains(i);
    let map_res = |i: usize| match is_error(&i) {
        true => Err(i.to_string()),
        false => Ok(i),
    };

    let result: Result<Vec<_>, _> = input
        .clone()
        .into_par()
        .iteration_order(IterationOrder::Arbitrary)
        .flat_map(flat_map)
        .filter(filter)
        .map(map)
        .filter_map(filter_map)
        .map(map2)
        .map_while_ok(map_res)
        .collect();

    let result = result.map_err(|e| {
        let number = e.parse::<usize>().unwrap();
        is_error(&number)
    });
    assert_eq!(result, Err(true));
}

#[test]
fn map_while_ok_from_xap_chain_whilst_when_ok() {
    let input = 0..1024;
    let flat_map = |i: usize| [i, 1000 + i, 2000 + i];
    let filter = |i: &usize| i < &2000;
    let map = |i: usize| 2 * i;
    let filter_map = |i: usize| (i % 2 == 0).then_some(i);
    let map2 = |i: usize| i / 2;
    let map_res = |i: usize| match (11300..11350).contains(&i) {
        true => Err(i.to_string()),
        false => Ok(i),
    };

    let result: Result<Vec<_>, _> = input
        .clone()
        .into_par()
        .iteration_order(IterationOrder::Arbitrary)
        .flat_map(flat_map)
        .filter(filter)
        .map(map)
        .filter_map(filter_map)
        .map(map2)
        .take_while(|i| i < &777)
        .map_while_ok(map_res)
        .collect();

    assert!(result.is_ok());
    let result = result.unwrap();
    let all_satisfies_whilst = result.iter().all(|x| x < &777);
    assert!(all_satisfies_whilst);
}

#[test]
fn map_while_ok_from_xap_chain_whilst_when_err() {
    let input = 0..1024;
    let flat_map = |i: usize| [i, 1024 + i, 2048 + i];
    let filter = |i: &usize| i < &1024;
    let map = |i: usize| 2 * i;
    let filter_map = |i: usize| (i % 2 == 0).then_some(i);
    let map2 = |i: usize| i / 2;
    let is_error =
        |i: &usize| (300..350).contains(i) || (400..450).contains(i) || (500..550).contains(i);
    let map_res = |i: usize| match is_error(&i) {
        true => Err(i.to_string()),
        false => Ok(i),
    };

    let result: Result<Vec<_>, _> = input
        .clone()
        .into_par()
        .iteration_order(IterationOrder::Arbitrary)
        .flat_map(flat_map)
        .filter(filter)
        .map(map)
        .filter_map(filter_map)
        .map(map2)
        .take_while(|i| i < &777)
        .map_while_ok(map_res)
        .collect();

    let result = result.map_err(|e| {
        let number = e.parse::<usize>().unwrap();
        number < 777 && is_error(&number)
    });
    assert_eq!(result, Err(true));
}

#[test]
fn map_while_ok_from_xap_chain_whilst_when_err_out_of_reach() {
    let input = 0..1024;
    let flat_map = |i: usize| [i, 1024 + i, 2048 + i];
    let filter = |i: &usize| i < &1024;
    let map = |i: usize| 2 * i;
    let filter_map = |i: usize| (i % 2 == 0).then_some(i);
    let map2 = |i: usize| i / 2;
    let is_error = |i: &usize| (800..850).contains(i);
    let map_res = |i: usize| match is_error(&i) {
        true => Err(i.to_string()),
        false => Ok(i),
    };

    let result: Result<Vec<_>, _> = input
        .clone()
        .into_par()
        .iteration_order(IterationOrder::Arbitrary)
        .flat_map(flat_map)
        .filter(filter)
        .map(map)
        .filter_map(filter_map)
        .map(map2)
        .take_while(|i| i < &777)
        .map_while_ok(map_res)
        .collect();

    assert!(result.is_ok());
    let result = result.unwrap();
    let all_satisfies_whilst = result.iter().all(|x| x < &777);
    assert!(all_satisfies_whilst);
}
