use serde::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Location {
    pub(crate) x: f64,
    pub(crate) y: f64,
}

const MIN_CITIES: usize = 5;
const MAX_CITIES: usize = 200;

pub fn locations(num_cities: u32) -> Result<JsValue, JsValue> {
    let num_cities = clamp_num_cities(num_cities);
    let locations: Vec<Location> = (0..num_cities).map(location_for).collect();
    serde_wasm_bindgen::to_value(&locations)
        .map_err(|e| JsValue::from_str(&format!("failed to serialize locations: {e}")))
}

pub fn clamp_num_cities(num_cities: u32) -> usize {
    (num_cities as usize).clamp(MIN_CITIES, MAX_CITIES)
}

pub fn location_for(idx: usize) -> Location {
    // Use a deterministic spiral-like layout for any requested city count.
    let t = idx as f64;
    let theta = t * 0.618_033_988_749_894_9 * core::f64::consts::TAU;
    let radius = 8.0 + 2.4 * t.sqrt();
    Location {
        x: radius * theta.cos(),
        y: radius * theta.sin(),
    }
}
