export const SEQUENTIAL_CODE = `let mut rng = SmallRng::seed_from_u64(seed);
(0..iterations)
    .map(|_| create_tour(&mut rng, locations))
    .min_by_key(|x| OrderedFloat::from(x.distance))`;

export const PARALLEL_CODE = `(0..iterations)
    .into_par()
    .use_new(|t| SmallRng::seed_from_u64(seed + t as u64))
    .map(|rng, _| create_tour(rng, locations))
    .min_by_key(|x| OrderedFloat::from(x.distance))`;

export const SEQUENTIAL_HELP = `// random-number-generator to construct initial random tours
let mut rng = SmallRng::seed_from_u64(seed);

// we will construct & improve \`iterations\` tours
(0..iterations)

    // \`create_tour\`constructs a random tour and locally-optimizes within 2-opt
    .map(|_| create_tour(&mut rng, locations))

    // among all created tours, we pick the one with minimum distance
    .min_by_key(|x| OrderedFloat::from(x.distance))`;

export const PARALLEL_HELP = `// we will construct & improve \`iterations\` tours
(0..iterations)

    // convert the iterator into parallel iterator
    .into_par()

    // \`use_new\` enables mutable variables in parallel computations
    // each thread will have its own random number generator
    // \`t\` here is the thread index, with value in (0..num_threads)
    .use_new(|t| SmallRng::seed_from_u64(seed + t as u64))

    // \`create_tour\`constructs a random tour and locally-optimizes within 2-opt
    .map(|rng, _| create_tour(rng, locations))

    // among all created tours, we pick the one with minimum distance
    .min_by_key(|x| OrderedFloat::from(x.distance))`;
