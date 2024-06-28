use orx_parallel::*;

pub(crate) fn test_string_owned<P>(par: P, input_len: usize)
where
    P: ParIter<Item = String>,
{
    let result = par
        .map(|x| {
            let len = x.len();
            (x, len)
        })
        .filter(|x| x.1 > 1)
        .flat_map(|x| {
            (0..(x.1.min(2) - 1))
                .map(|_| x.0.clone())
                .collect::<Vec<_>>()
        })
        .filter_map(|x| x.parse::<usize>().ok())
        .filter(|x| x % 2 == 0)
        .map(|x| x * 2)
        .collect_vec();

    let expected: Vec<_> = (0..input_len)
        .filter(|x| x > &9 && x % 2 == 0)
        .map(|x| x * 2)
        .collect();

    assert_eq!(result, expected);
}

pub(crate) fn test_string_ref<'a, P>(par: P, input_len: usize)
where
    P: ParIter<Item = &'a String>,
{
    let result = par
        .map(|x| (x, x.len()))
        .filter(|x| x.1 > 1)
        .flat_map(|x| (0..(x.1.min(2) - 1)).map(|_| x.0).collect::<Vec<_>>())
        .filter_map(|x| x.parse::<usize>().ok())
        .filter(|x| x % 2 == 0)
        .map(|x| x * 2)
        .collect_vec();

    let expected: Vec<_> = (0..input_len)
        .filter(|x| x > &9 && x % 2 == 0)
        .map(|x| x * 2)
        .collect();

    assert_eq!(result, expected);
}

pub(crate) fn test_numbers_owned<P>(par: P, input_len: usize)
where
    P: ParIter<Item = usize>,
{
    let result = par
        .map(|x| x.to_string())
        .map(|x| {
            let len = x.len();
            (x, len)
        })
        .filter(|x| x.1 > 1)
        .flat_map(|x| {
            (0..(x.1.min(2) - 1))
                .map(|_| x.0.clone())
                .collect::<Vec<_>>()
        })
        .filter_map(|x| x.parse::<usize>().ok())
        .filter(|x| x % 2 == 0)
        .map(|x| x * 2)
        .collect_vec();

    let expected: Vec<_> = (0..input_len)
        .filter(|x| x > &9 && x % 2 == 0)
        .map(|x| x * 2)
        .collect();

    assert_eq!(result, expected);
}

pub(crate) fn test_numbers_ref<'a, P>(par: P, input_len: usize)
where
    P: ParIter<Item = &'a usize>,
{
    let result = par
        .map(|x| x.to_string())
        .map(|x| {
            let len = x.len();
            (x, len)
        })
        .filter(|x| x.1 > 1)
        .flat_map(|x| {
            (0..(x.1.min(2) - 1))
                .map(|_| x.0.clone())
                .collect::<Vec<_>>()
        })
        .filter_map(|x| x.parse::<usize>().ok())
        .filter(|x| x % 2 == 0)
        .map(|x| x * 2)
        .collect_vec();

    let expected: Vec<_> = (0..input_len)
        .filter(|x| x > &9 && x % 2 == 0)
        .map(|x| x * 2)
        .collect();

    assert_eq!(result, expected);
}
