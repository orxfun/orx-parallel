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

    let result = input
        .into_par()
        .flat_map(flat_map)
        .filter(filter)
        .map(map)
        .filter_map(filter_map)
        .map(map2)
        .map_while_ok(map_res)
        .reduce(|a, b| a + b);
    let expected = Ok((0..1024)
        .flat_map(flat_map)
        .filter(filter)
        .map(map)
        .filter_map(filter_map)
        .map(map2)
        .reduce(|a, b| a + b));

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

    let result = input
        .clone()
        .into_par()
        .flat_map(flat_map)
        .filter(filter)
        .map(map)
        .filter_map(filter_map)
        .map(map2)
        .map_while_ok(map_res)
        .reduce(|a, b| a + b);

    let result = result.map_err(|e| {
        let number = e.parse::<usize>().unwrap();
        is_error(&number)
    });
    assert_eq!(result, Err(true));
}
