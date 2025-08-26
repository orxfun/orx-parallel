use orx_parallel::*;

#[test]
fn map_while_ok_from_map_when_ok() {
    let input = 1..1025;
    let map = |i: usize| i - 1;
    let map_res = |i: usize| match (1300..1350).contains(&i) {
        true => Err(i.to_string()),
        false => Ok(i),
    };

    let result: Result<Vec<_>, _> = input
        .into_par()
        .map(map)
        .map(map_res)
        .into_fallible_result()
        .collect();
    let expected = Ok((0..1024).collect::<Vec<_>>());

    assert_eq!(result, expected);
}

#[test]
fn map_while_ok_from_map_when_error() {
    let input = 1..1025;
    let map = |i: usize| i - 1;
    let is_error =
        |i: &usize| (300..350).contains(i) || (400..450).contains(i) || (500..550).contains(i);
    let map_res = |i: usize| match is_error(&i) {
        true => Err(i.to_string()),
        false => Ok(i),
    };

    let result: Result<Vec<_>, _> = input
        .into_par()
        .map(map)
        .map(map_res)
        .into_fallible_result()
        .collect();

    let result = result.map_err(|e| {
        let number = e.parse::<usize>().unwrap();
        is_error(&number)
    });
    assert_eq!(result, Err(true));
}
