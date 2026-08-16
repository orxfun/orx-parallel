use crate::{alg::Method, input::InputVariant};
use orx_criterion::Experiment;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

const GRAVITY: f64 = 1.0;
const SOFTENING: f64 = 0.01;
const TIME_STEP: f64 = 0.01;

#[derive(Clone, Copy, Debug)]
pub struct Body {
    pub mass: f64,
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SimulationResult {
    pub num_bodies: usize,
    pub position_checksum: f64,
    pub total_energy: f64,
}

fn create_system(seed: u64, num_bodies: usize) -> Vec<Body> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    (0..num_bodies)
        .map(|_| Body {
            mass: rng.random_range(0.5..2.0),
            x: rng.random_range(-100.0..100.0),
            y: rng.random_range(-100.0..100.0),
            vx: rng.random_range(-0.1..0.1),
            vy: rng.random_range(-0.1..0.1),
        })
        .collect()
}

fn integrate_body(body: &Body, bodies: &[Body]) -> Body {
    let mut ax = 0.0;
    let mut ay = 0.0;

    for other in bodies {
        let dx = other.x - body.x;
        let dy = other.y - body.y;
        let distance_squared = dx * dx + dy * dy + SOFTENING * SOFTENING;
        let inverse_distance = distance_squared.sqrt().recip();
        let inverse_distance_cubed = inverse_distance * inverse_distance * inverse_distance;
        let scale = GRAVITY * other.mass * inverse_distance_cubed;
        ax += dx * scale;
        ay += dy * scale;
    }

    Body {
        mass: body.mass,
        x: body.x + body.vx * TIME_STEP + 0.5 * ax * TIME_STEP * TIME_STEP,
        y: body.y + body.vy * TIME_STEP + 0.5 * ay * TIME_STEP * TIME_STEP,
        vx: body.vx + ax * TIME_STEP,
        vy: body.vy + ay * TIME_STEP,
    }
}

fn total_energy(bodies: &[Body]) -> f64 {
    let kinetic: f64 = bodies
        .iter()
        .map(|body| 0.5 * body.mass * (body.vx * body.vx + body.vy * body.vy))
        .sum();

    let potential: f64 = bodies
        .iter()
        .enumerate()
        .flat_map(|(i, first)| bodies.iter().skip(i + 1).map(move |second| (first, second)))
        .map(|(first, second)| {
            let dx = second.x - first.x;
            let dy = second.y - first.y;
            let distance = (dx * dx + dy * dy + SOFTENING * SOFTENING).sqrt();
            -GRAVITY * first.mass * second.mass / distance
        })
        .sum();

    kinetic + potential
}

fn summarize(bodies: &[Body]) -> SimulationResult {
    let position_checksum = bodies
        .iter()
        .map(|body| body.mass * (body.x + body.y))
        .sum();

    SimulationResult {
        num_bodies: bodies.len(),
        position_checksum,
        total_energy: total_energy(bodies),
    }
}

pub struct Exp;

impl Experiment for Exp {
    type InputFactors = InputVariant;
    type AlgFactors = Method;
    type Input = Vec<Body>;
    type Output = SimulationResult;

    fn input(&mut self, input_variant: &Self::InputFactors) -> Self::Input {
        create_system(0x4E_B0_D1, input_variant.num_bodies)
    }

    fn execute(
        &mut self,
        input_variant: &Self::InputFactors,
        alg_variant: &Self::AlgFactors,
        input: &Self::Input,
    ) -> Self::Output {
        match alg_variant {
            Method::Seq => run_sequential(input, input_variant.steps),
            Method::Rayon => run_rayon(input, input_variant.steps),
            Method::OrxOnce | Method::OrxBasic | Method::OrxRayon => {
                run_orx(input, input_variant.steps)
            }
        }
    }

    fn validate_output(
        &self,
        input_variant: &Self::InputFactors,
        _input: &Self::Input,
        output: &Self::Output,
    ) {
        assert_eq!(output.num_bodies, input_variant.num_bodies);
        assert!(output.position_checksum.is_finite());
        assert!(output.total_energy.is_finite());
    }
}

fn run_sequential(initial: &[Body], steps: usize) -> SimulationResult {
    let mut bodies = initial.to_vec();
    for _ in 0..steps {
        let next: Vec<_> = bodies
            .iter()
            .map(|body| integrate_body(body, &bodies))
            .collect();
        bodies = next;
    }
    summarize(&bodies)
}

fn run_rayon(initial: &[Body], steps: usize) -> SimulationResult {
    use rayon::prelude::*;

    let mut bodies = initial.to_vec();
    for _ in 0..steps {
        let next: Vec<_> = bodies
            .par_iter()
            .map(|body| integrate_body(body, &bodies))
            .collect();
        bodies = next;
    }
    summarize(&bodies)
}

fn run_orx(initial: &[Body], steps: usize) -> SimulationResult {
    use orx_parallel::*;

    let mut bodies = initial.to_vec();
    for _ in 0..steps {
        let next: Vec<_> = bodies
            .par()
            .map(|body| integrate_body(body, &bodies))
            .collect();
        bodies = next;
    }
    summarize(&bodies)
}
