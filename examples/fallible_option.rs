use orx_parallel::*;

fn parse_line_total(row: &str) -> Option<usize> {
    let mut parts = row.split(',');

    let qty = parts.next()?.parse::<usize>().ok()?;
    let unit_price = parts.next()?.parse::<usize>().ok()?;

    Some(qty * unit_price)
}

fn main() {
    let good_rows = vec!["2,1500", "1,2300", "4,499", "5,1100"];
    let total = good_rows
        .par()
        .map(|row| parse_line_total(row))
        .into_optional()
        // After `into_optional`, we keep writing the computation for the success path only,
        // just like using `?` lets us focus on the `Some` value instead of handling `None`.
        .filter(|line_total| *line_total >= 2_000)
        // `sum` also works directly on successful line totals; missing or invalid rows stay implicit.
        .sum();

    assert_eq!(total, Some(2 * 1500 + 2300 + 5 * 1100));

    // The computation short-circuits and stops immediately once any worker observes `None`.
    let bad_rows = vec!["2,1500", "1,2300", "x,499", "5,1100"];
    let total = bad_rows
        .par()
        .map(|row| parse_line_total(row))
        .into_optional()
        .filter(|line_total| *line_total >= 2_000)
        .sum();

    assert_eq!(total, None);
}
