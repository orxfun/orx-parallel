use serde::Serialize;

const MIN_CITIES: usize = 5;
const MAX_CITIES: usize = 200;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Location {
    pub x: f64,
    pub y: f64,
}

impl Location {
    pub fn distance_to(self, other: Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn tour_distance(locations: &[Location], tour: &[usize]) -> f64 {
        match (tour.first(), tour.last()) {
            (Some(&first), Some(&last)) => {
                let middle_distance: f64 = tour
                    .windows(2)
                    .map(|w| locations[w[0]].distance_to(locations[w[1]]))
                    .sum();
                let closing_distance = locations[last].distance_to(locations[first]);
                middle_distance + closing_distance
            }
            _ => 0.0,
        }
    }
}

pub fn locations(num_cities: u32) -> Vec<Location> {
    let num_cities = clamp_num_cities(num_cities);
    (0..num_cities).map(location_for).collect()
}

pub fn clamp_num_cities(num_cities: u32) -> usize {
    (num_cities as usize).clamp(MIN_CITIES, MAX_CITIES)
}

pub fn location_for(idx: usize) -> Location {
    let sx = split_mix_64((idx as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    let sy = split_mix_64((idx as u64).wrapping_mul(0xD1B5_4A32_D192_ED03));

    let x = 100.0 * to_unit_f64(sx) - 50.0;
    let y = 100.0 * to_unit_f64(sy) - 50.0;
    Location { x, y }
}

fn split_mix_64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn to_unit_f64(x: u64) -> f64 {
    let v = x >> 11;
    (v as f64) * (1.0 / ((1u64 << 53) as f64))
}
