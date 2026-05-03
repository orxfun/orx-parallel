use orx_parallel::*;

fn parse_line_total(row: &str) -> Result<usize, String> {
    let mut parts = row.split(',');

    let qty = parts
        .next()
        .ok_or("missing quantity")?
        .parse::<usize>()
        .map_err(|_| "invalid quantity".to_string())?;

    let unit_price = parts
        .next()
        .ok_or("missing unit price")?
        .parse::<usize>()
        .map_err(|_| "invalid unit price".to_string())?;

    Ok(qty * unit_price)
}

fn main() {
    let good_rows = vec!["2,1500", "1,2300", "4,499", "5,1100"];
    let total = good_rows
        .par()
        .map(|row| parse_line_total(row))
        .into_fallible()
        // After `into_fallible`, we keep writing the computation for the success path only,
        // just like using `?` lets us focus on the `Ok` value instead of the error variant.
        .filter(|line_total| *line_total >= 2_000)
        // `sum` also works directly on successful line totals; error handling stays implicit.
        .sum();

    assert_eq!(total, Ok(2 * 1500 + 2300 + 5 * 1100));

    // The computation short-circuits and stops immediately once any worker observes an error.
    let bad_rows = vec!["2,1500", "1,2300", "x,499", "5,1100"];
    let total = bad_rows
        .par()
        .map(|row| parse_line_total(row))
        .into_fallible()
        .filter(|line_total| *line_total >= 2_000)
        .sum();

    assert!(total.is_err());
}
