use orx_parallel::*;

#[test]
fn map_while_ok_from_xap_flat_map_when_ok() {
    let input = 0..1024;
    let flat_map = |i: usize| [i, 1000 + i, 2000 + i];
    let map_res = |i: usize| match (11300..11350).contains(&i) {
        true => Err(i.to_string()),
        false => Ok(i),
    };

    let result = input
        .into_par()
        .flat_map(flat_map)
        .map_while_ok(map_res)
        .reduce(|a, b| a + b);
    let expected = Ok(Some((0..1024).flat_map(flat_map).sum()));

    assert_eq!(result, expected);
}

#[test]
fn map_while_ok_from_xap_flat_map_when_ok_but_none() {
    let input = 0..1024;
    let flat_map = |_: usize| [];
    let map_res = |i: usize| match (11300..11350).contains(&i) {
        true => Err(i.to_string()),
        false => Ok(i),
    };

    let result = input
        .into_par()
        .flat_map(flat_map)
        .map_while_ok(map_res)
        .reduce(|a, b| a + b);
    let expected = Ok(None);

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

    let result = input
        .into_par()
        .flat_map(flat_map)
        .map_while_ok(map_res)
        .reduce(|a, b| a + b);

    let result = result.map_err(|e| {
        let number = e.parse::<usize>().unwrap();
        is_error(&number)
    });
    assert_eq!(result, Err(true));
}
