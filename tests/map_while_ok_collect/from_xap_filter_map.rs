use orx_parallel::*;

#[test]
fn map_while_ok_from_xap_filter_map_when_ok() {
    let input = 0..1024;
    let filter_map = |i: usize| (i % 2 == 1).then_some(i);
    let map_res = |i: usize| match (1300..1350).contains(&i) {
        true => Err(i.to_string()),
        false => Ok(i),
    };

    let result: Result<Vec<_>, _> = input
        .into_par()
        .filter_map(filter_map)
        .map(map_res)
        .into_fallible()
        .collect();
    let expected = Ok((0..1024).filter_map(filter_map).collect::<Vec<_>>());

    assert_eq!(result, expected);
}

#[test]
fn map_while_ok_from_xap_filter_map_when_error() {
    let input = 0..1024;
    let filter_map = |i: usize| (i % 2 == 1).then_some(i);
    let is_error =
        |i: &usize| (300..350).contains(i) || (400..450).contains(i) || (500..550).contains(i);
    let map_res = |i: usize| match is_error(&i) {
        true => Err(i.to_string()),
        false => Ok(i),
    };

    let result: Result<Vec<_>, _> = input
        .into_par()
        .filter_map(filter_map)
        .map(map_res)
        .into_fallible()
        .collect();

    let result = result.map_err(|e| {
        let number = e.parse::<usize>().unwrap();
        is_error(&number)
    });
    assert_eq!(result, Err(true));
}

#[test]
fn map_while_ok_from_xap_filter_map_whilst_when_ok() {
    let input = 0..1024;
    let filter_map = |i: usize| (i % 2 == 1).then_some(i);
    let map_res = |i: usize| match (1300..1350).contains(&i) {
        true => Err(i.to_string()),
        false => Ok(i),
    };

    let result: Result<Vec<_>, _> = input
        .into_par()
        .filter_map(filter_map)
        .take_while(|i| i < &777)
        .map(map_res)
        .into_fallible()
        .collect();
    let expected = Ok((0..1024)
        .filter_map(filter_map)
        .take_while(|i| i < &777)
        .collect::<Vec<_>>());

    assert_eq!(result, expected);
}

#[test]
fn map_while_ok_from_xap_filter_map_whilst_when_error() {
    let input = 0..1024;
    let filter_map = |i: usize| (i % 2 == 1).then_some(i);
    let is_error =
        |i: &usize| (300..350).contains(i) || (400..450).contains(i) || (500..550).contains(i);
    let map_res = |i: usize| match is_error(&i) {
        true => Err(i.to_string()),
        false => Ok(i),
    };

    let result: Result<Vec<_>, _> = input
        .into_par()
        .filter_map(filter_map)
        .take_while(|i| i < &777)
        .map(map_res)
        .into_fallible()
        .collect();

    let result = result.map_err(|e| {
        let number = e.parse::<usize>().unwrap();
        number < 777 && is_error(&number)
    });
    assert_eq!(result, Err(true));
}

#[test]
fn map_while_ok_from_xap_filter_map_whilst_when_error_out_of_reach() {
    let input = 0..1024;
    let filter_map = |i: usize| (i % 2 == 1).then_some(i);
    let is_error = |i: &usize| (800..850).contains(i);
    let map_res = |i: usize| match is_error(&i) {
        true => Err(i.to_string()),
        false => Ok(i),
    };

    let result: Result<Vec<_>, _> = input
        .into_par()
        .filter_map(filter_map)
        .take_while(|i| i < &777)
        .map(map_res)
        .into_fallible()
        .collect();

    let expected = Ok((0..1024)
        .filter_map(filter_map)
        .take_while(|i| i < &777)
        .collect::<Vec<_>>());

    assert_eq!(result, expected);
}
