use orx_parallel::*;
use std::cmp::Ordering;

struct Location {
    x: i64,
    y: i64,
}

fn distance(a: &Location, b: &Location) -> f64 {
    let x = (a.x - b.x) * (a.x - b.x);
    let y = (a.y - b.y) * (a.y - b.y);
    ((x + y) as f64).sqrt()
}

fn generate(locations: &[Location]) -> (Vec<usize>, f64) {
    let tour: Vec<_> = (0..locations.len()).collect();
    let links = tour.iter().zip(tour.iter().skip(1));
    let distance = links
        .map(|(i, j)| distance(&locations[*i], &locations[*j]))
        .sum();
    (tour, distance)
}

fn main() {
    let locations = vec![
        Location { x: 3, y: 7 },
        Location { x: 9, y: 5 },
        Location { x: 3, y: 0 },
        Location { x: 5, y: -3 },
        Location { x: 12, y: 4 },
        Location { x: 8, y: 9 },
        Location { x: 6, y: 17 },
        Location { x: 0, y: 71 },
    ];

    let best_tour = (0..100)
        .par()
        .map(|_| generate(&locations))
        .min_by(|x, y| match x.1 < y.1 {
            true => Ordering::Less,
            false => Ordering::Greater,
        })
        .map(|x| x.1);
}
