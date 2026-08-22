use orx_parallel::*;

fn fibonacci_term(index: usize) -> u64 {
    let mut previous = 0;
    let mut current = 1;

    for _ in 0..index {
        (previous, current) = (current, previous + current);
    }

    previous
}

pub fn calculate_fibonacci(workload: usize, num_threads: usize) -> u64 {
    (0..workload)
        .par()
        .num_threads(num_threads)
        .map(|index| fibonacci_term(index % 40))
        .sum()
}

fn is_prime(candidate: usize) -> bool {
    if candidate < 2 {
        return false;
    }

    if candidate == 2 {
        return true;
    }

    if candidate.is_multiple_of(2) {
        return false;
    }

    let mut divisor = 3;
    while divisor <= candidate / divisor {
        if candidate.is_multiple_of(divisor) {
            return false;
        }
        divisor += 2;
    }
    true
}

pub fn count_primes(limit: usize, num_threads: usize) -> usize {
    (2..limit)
        .par()
        .num_threads(num_threads)
        .filter(|&candidate| is_prime(candidate))
        .count()
        + usize::from(is_prime(limit))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_fibonacci_terms() {
        assert_eq!(calculate_fibonacci(6, 2), 12);
    }

    #[test]
    fn counts_primes() {
        assert_eq!(count_primes(10, 2), 4);
    }
}
