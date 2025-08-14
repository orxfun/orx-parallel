use orx_parallel::*;

#[test]
fn map_while_ok_from_par_when_ok() {
    let input = 0..1024;
    let map = |i: usize| match (1300..1350).contains(&i) {
        true => Err(i.to_string()),
        false => Ok(i),
    };

    let result: Result<Vec<_>, _> = input.into_par().map_while_ok(map).collect();
    let expected = Ok((0..1024).collect::<Vec<_>>());

    assert_eq!(result, expected);
}

#[test]
fn map_while_ok_from_par_when_error() {
    let input = 0..1024;
    let map = |i: usize| match (300..350).contains(&i)
        || (400..450).contains(&i)
        || (500..550).contains(&i)
    {
        true => Err(i.to_string()),
        false => Ok(i),
    };

    let result: Result<Vec<_>, _> = input.into_par().map_while_ok(map).collect();
    let expected = Err(300.to_string()); // must stop at the first error

    assert_eq!(result, expected);
}
