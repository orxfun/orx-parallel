pub fn div_ceil(number: usize, divider: usize) -> usize {
    let x = number / divider;
    let remainder = number - x * divider;
    x + if remainder > 0 { 1 } else { 0 }
}
