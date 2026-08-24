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
        .map(|index| fibonacci_term(index))
        .sum()
}

const MAX_MANDELBROT_ITERATIONS: u64 = 10000;

fn mandelbrot_escape_iterations(point_index: usize, workload: usize) -> u64 {
    let width = (workload as f64).sqrt().ceil() as usize;
    let height = workload.div_ceil(width);
    let x = point_index % width;
    let y = point_index / width;

    let real = -2.0 + 3.0 * x as f64 / width.saturating_sub(1).max(1) as f64;
    let imaginary = -1.5 + 3.0 * y as f64 / height.saturating_sub(1).max(1) as f64;
    let (mut z_real, mut z_imaginary) = (0.0, 0.0);

    for iteration in 1..=MAX_MANDELBROT_ITERATIONS {
        (z_real, z_imaginary) = (
            z_real * z_real - z_imaginary * z_imaginary + real,
            2.0 * z_real * z_imaginary + imaginary,
        );

        if z_real * z_real + z_imaginary * z_imaginary > 4.0 {
            return iteration;
        }
    }

    MAX_MANDELBROT_ITERATIONS
}

pub fn mandelbrot_checksum(workload: usize, num_threads: usize) -> u64 {
    (0..workload)
        .par()
        .num_threads(num_threads)
        .map(|point_index| mandelbrot_escape_iterations(point_index, workload))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_fibonacci_terms() {
        assert_eq!(calculate_fibonacci(6, 2), 12);
    }

    #[test]
    fn calculates_mandelbrot_checksum() {
        assert_eq!(mandelbrot_checksum(4, 2), 6);
    }
}
