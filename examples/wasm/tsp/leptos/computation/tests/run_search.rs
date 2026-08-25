use computation::*;

#[test]
fn search_produces_valid_tour() {
    let locations = create_locations(123, 20);
    let iterations = 32;
    let seed = 12345;

    let output = run_search(iterations, seed, 2, 1, &locations);

    let best = output.expect("search should find a tour");

    // Validate tour is complete and contains all cities
    assert_eq!(best.tour.len(), locations.len());
    let mut seen = vec![false; locations.len()];
    for &city in &best.tour {
        assert!(city < locations.len(), "tour contains invalid city index");
        assert!(!seen[city], "tour contains duplicate city");
        seen[city] = true;
    }

    // Validate distance is correct
    let calculated_distance = Location::tour_distance(&locations, &best.tour);
    assert_eq!(best.distance, calculated_distance);
}
