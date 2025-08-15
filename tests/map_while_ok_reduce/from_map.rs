use orx_parallel::*;

#[test]
fn map_while_ok_from_map_when_ok() {
    let map_res = |i: usize| match (1300..1350).contains(&i) {
        true => Err(i.to_string()),
        false => Ok(i),
    };

    let result = (1..1025)
        .into_par()
        .map(|i| i - 1)
        .map(map_res)
        .into_fallible()
        .reduce(|a, b| a + b);
    let expected = Ok(Some((0..1024).sum::<usize>()));

    assert_eq!(result, expected);

    let result = (1..1)
        .into_par()
        .map(|i| i - 1)
        .map(map_res)
        .into_fallible()
        .reduce(|a, b| a + b);
    let expected = Ok(None);

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

    let result = input
        .into_par()
        .map(map)
        .map(map_res)
        .into_fallible()
        .reduce(|a, b| a + b);

    let result = result.map_err(|e| {
        let number = e.parse::<usize>().unwrap();
        is_error(&number)
    });
    assert_eq!(result, Err(true));
}
