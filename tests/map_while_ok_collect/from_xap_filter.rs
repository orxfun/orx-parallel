use orx_parallel::*;

#[test]
fn map_while_ok_from_xap_filter_when_ok() {
    let input = 0..1024;
    let filter = |i: &usize| i % 2 == 1;
    let map_res = |i: usize| match (1300..1350).contains(&i) {
        true => Err(i.to_string()),
        false => Ok(i),
    };

    let result: Result<Vec<_>, _> = input
        .into_par()
        .filter(filter)
        .map_while_ok(map_res)
        .collect();
    let expected = Ok((0..1024).filter(filter).collect::<Vec<_>>());

    assert_eq!(result, expected);
}

#[test]
fn map_while_ok_from_xap_filter_when_error() {
    let input = 0..1024;
    let filter = |i: &usize| i % 2 == 1;
    let is_error =
        |i: &usize| (300..350).contains(i) || (400..450).contains(i) || (500..550).contains(i);
    let map_res = |i: usize| match is_error(&i) {
        true => Err(i.to_string()),
        false => Ok(i),
    };

    let result: Result<Vec<_>, _> = input
        .into_par()
        .filter(filter)
        .map_while_ok(map_res)
        .collect();

    let result = result.map_err(|e| {
        let number = e.parse::<usize>().unwrap();
        is_error(&number)
    });
    assert_eq!(result, Err(true));
}

#[test]
fn map_while_ok_from_xap_filter_whilst_when_ok() {
    let input = 0..1024;
    let filter = |i: &usize| i % 2 == 1;
    let map_res = |i: usize| match (1300..1350).contains(&i) {
        true => Err(i.to_string()),
        false => Ok(i),
    };

    let result: Result<Vec<_>, _> = input
        .into_par()
        .filter(filter)
        .take_while(|i| i < &777)
        .map_while_ok(map_res)
        .collect();
    let expected = Ok((0..1024)
        .filter(filter)
        .take_while(|i| i < &777)
        .collect::<Vec<_>>());

    assert_eq!(result, expected);
}

#[test]
fn map_while_ok_from_xap_filter_whilst_when_err() {
    let input = 0..1024;
    let filter = |i: &usize| i % 2 == 1;
    let is_error =
        |i: &usize| (300..350).contains(i) || (400..450).contains(i) || (500..550).contains(i);
    let map_res = |i: usize| match is_error(&i) {
        true => Err(i.to_string()),
        false => Ok(i),
    };

    let result: Result<Vec<_>, _> = input
        .into_par()
        .filter(filter)
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
fn map_while_ok_from_xap_filter_whilst_when_err_out_of_reach() {
    let input = 0..1024;
    let filter = |i: &usize| i % 2 == 1;
    let is_error = |i: &usize| (800..850).contains(i);
    let map_res = |i: usize| match is_error(&i) {
        true => Err(i.to_string()),
        false => Ok(i),
    };

    let result: Result<Vec<_>, _> = input
        .into_par()
        .filter(filter)
        .take_while(|i| i < &777)
        .map_while_ok(map_res)
        .collect();

    let expected = Ok((0..1024)
        .filter(filter)
        .take_while(|i| i < &777)
        .collect::<Vec<_>>());

    assert_eq!(result, expected);
}
