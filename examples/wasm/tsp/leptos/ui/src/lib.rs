mod canvas_view;
mod code_card;
mod controls;
mod status;

use canvas_view::{CanvasView, draw_scene};
use code_card::CodeCard;
use computation::{Location, create_locations};
use controls::ControlsSection;
use gloo_timers::{callback::Interval, future::TimeoutFuture};
use js_sys::Date;
use leptos::html;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use status::StatusSection;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::{JsValue, wasm_bindgen};
use wasm_bindgen_futures::JsFuture;
use wasm_bindings::RunResult;

const MIN_CITIES: u32 = 5;
const MAX_CITIES: u32 = 200;
const MIN_THREADS: u32 = 1;
const MAX_THREADS: u32 = 16;

const SEQUENTIAL_CODE: &str = "let mut rng = SmallRng::seed_from_u64(seed);\n(0..iterations)\n    .map(|_| create_tour(&mut rng, locations))\n    .min_by_key(|x| OrderedFloat::from(x.distance))";

const PARALLEL_CODE: &str = "(0..iterations)\n    .into_par()\n    .use_new(|t| SmallRng::seed_from_u64(seed + t as u64))\n    .map(|rng, _| create_tour(rng, locations))\n    .min_by_key(|x| OrderedFloat::from(x.distance))";

const SEQUENTIAL_HELP: &str = "// random-number-generator to construct initial random tours\nlet mut rng = SmallRng::seed_from_u64(seed);\n\n// we will construct & improve `iterations` tours\n(0..iterations)\n\n    // `create_tour` constructs a random tour and locally optimizes within 2-opt\n    .map(|_| create_tour(&mut rng, locations))\n\n    // among all created tours, we pick the one with minimum distance\n    .min_by_key(|x| OrderedFloat::from(x.distance))";

const PARALLEL_HELP: &str = "// we will construct & improve `iterations` tours\n(0..iterations)\n\n    // convert the iterator into parallel iterator\n    .into_par()\n\n    // `use_new` enables mutable variables in parallel computations\n    // each thread will have its own random number generator\n    // `t` here is the thread index, with value in (0..num_threads)\n    .use_new(|t| SmallRng::seed_from_u64(seed + t as u64))\n\n    // `create_tour` constructs a random tour and locally optimizes within 2-opt\n    .map(|rng, _| create_tour(rng, locations))\n\n    // among all created tours, we pick the one with minimum distance\n    .min_by_key(|x| OrderedFloat::from(x.distance))";

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
enum SearchMode {
    Parallel,
    Sequential,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RunSettings {
    mode: SearchMode,
    iterations: u32,
    threads: u32,
    chunk_size: u32,
    seed: u64,
    num_cities: u32,
}

#[derive(Clone)]
struct UiState {
    canvas_ref: NodeRef<html::Canvas>,
    status: RwSignal<String>,
    iterations: RwSignal<u32>,
    threads: RwSignal<u32>,
    chunk_size: RwSignal<u32>,
    seed: RwSignal<u64>,
    num_cities: RwSignal<u32>,
    points: RwSignal<Vec<Location>>,
    best_so_far: RwSignal<Option<RunResult>>,
    best_distance: RwSignal<String>,
    elapsed: RwSignal<String>,
    ips: RwSignal<String>,
    is_running: RwSignal<bool>,
    run_mode: RwSignal<SearchMode>,
    run_subtitle: RwSignal<String>,
    run_elapsed: RwSignal<String>,
    run_started_at_ms: RwSignal<f64>,
    run_ticker: Rc<RefCell<Option<Interval>>>,
}

#[wasm_bindgen(js_namespace = globalThis)]
unsafe extern "C" {
    fn runSearchOnce(settings: JsValue) -> js_sys::Promise;
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start_app() {
    console_error_panic_hook::set_once();
    mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let status = RwSignal::new("Initializing...".to_string());
    let iterations = RwSignal::new(10_000_u32);
    let threads = RwSignal::new(4_u32);
    let chunk_size = RwSignal::new(0_u32);
    let seed = RwSignal::new(42_u64);
    let num_cities = RwSignal::new(50_u32);
    let points = RwSignal::new(create_locations(
        seed.get_untracked(),
        num_cities.get_untracked(),
    ));
    let best_so_far = RwSignal::new(None::<RunResult>);
    let best_distance = RwSignal::new("-".to_string());
    let elapsed = RwSignal::new("-".to_string());
    let ips = RwSignal::new("-".to_string());
    let is_running = RwSignal::new(false);
    let run_mode = RwSignal::new(SearchMode::Parallel);
    let run_subtitle =
        RwSignal::new("Working through candidate tours. Larger runs can take a while.".to_string());
    let run_elapsed = RwSignal::new("Elapsed: 0.0 s".to_string());
    let run_started_at_ms = RwSignal::new(0.0_f64);
    let run_ticker = Rc::new(RefCell::new(None::<Interval>));
    let canvas_ref = NodeRef::<html::Canvas>::new();

    let ui = UiState {
        canvas_ref,
        status,
        iterations,
        threads,
        chunk_size,
        seed,
        num_cities,
        points,
        best_so_far,
        best_distance,
        elapsed,
        ips,
        is_running,
        run_mode,
        run_subtitle,
        run_elapsed,
        run_started_at_ms,
        run_ticker,
    };

    let city_node_color = read_css_color("--city-node");
    let tour_line_color = read_css_color("--tour-line");
    let canvas_background_color = read_css_color("--code-block-bg");

    view! {
        <main>
            <header class="hero">
                <a href="https://github.com/orxfun" target="_blank" rel="noreferrer">
                    <img
                        class="hero-logo"
                        src="https://avatars.githubusercontent.com/u/132661625?s=400&u=e13dbda1a79636fa7d02dd9ac8dfc02705694144&v=4"
                        alt="orx logo"
                    />
                </a>
                <div class="hero-copy">
                    <h1>Parallel computation in WASM</h1>
                    <p class="hero-kicker">A Leptos TSP demo with shared-memory wasm threads using orx-parallel.</p>
                </div>
            </header>

            <section class="intro-cards" aria-label="Example overview">
                <article class="intro-card">
                    <h2>Computation: Local Search for TSP</h2>
                    <p>
                        Randomly create tours and locally optimize with two-opt <code>Iterations</code> times, and return the best tour.
                    </p>
                </article>
                <article class="intro-card">
                    <h2>Frontend</h2>
                    <p>Leptos + Rust + Vite UI.</p>
                </article>
                <article class="intro-card">
                    <CodeCard title="Sequential Code" help_title="Sequential Code Breakdown" help_body=SEQUENTIAL_HELP code=SEQUENTIAL_CODE />
                </article>
                <article class="intro-card">
                    <CodeCard title="Parallel Code" help_title="Parallel Code Breakdown" help_body=PARALLEL_HELP code=PARALLEL_CODE />
                </article>
            </section>

            <ControlsSection ui=ui.clone() />

            <section class="card">
                <StatusSection ui=ui.clone() />
                <CanvasView
                    ui=ui.clone()
                    city_node_color=city_node_color.clone()
                    tour_line_color=tour_line_color.clone()
                    canvas_background_color=canvas_background_color.clone()
                />
            </section>
        </main>
    }
}

async fn run_search(ui: UiState, mode: SearchMode) {
    let settings = RunSettings {
        mode,
        iterations: ui.iterations.get_untracked(),
        threads: ui.threads.get_untracked(),
        chunk_size: ui.chunk_size.get_untracked(),
        seed: ui.seed.get_untracked(),
        num_cities: ui.num_cities.get_untracked(),
    };

    set_running_view(&ui, settings.mode, true);
    allow_running_overlay_to_render().await;
    ui.status.set(run_label(settings.mode).to_string());

    let request = match serde_wasm_bindgen::to_value(&settings) {
        Ok(value) => value,
        Err(err) => {
            ui.status
                .set(format!("Error: failed to serialize run settings: {err}"));
            set_running_view(&ui, settings.mode, false);
            return;
        }
    };

    let response = JsFuture::from(runSearchOnce(request)).await;

    match response {
        Ok(value) => match serde_wasm_bindgen::from_value::<RunResult>(value) {
            Ok(result) => {
                let RunResult {
                    best_tour,
                    best_distance,
                    iterations,
                    elapsed_ms,
                } = result;

                let should_replace = ui
                    .best_so_far
                    .get_untracked()
                    .as_ref()
                    .is_none_or(|best| best_distance < best.best_distance);

                if should_replace {
                    ui.best_so_far.set(Some(RunResult {
                        best_tour,
                        best_distance,
                        iterations,
                        elapsed_ms,
                    }));
                    draw_scene(
                        &ui.canvas_ref,
                        &ui.points.get_untracked(),
                        ui.best_so_far.get_untracked().as_ref(),
                        "#f59e0b",
                        "#1d4ed8",
                        "#0f172a",
                    );
                }

                update_stats(&ui, iterations, best_distance, elapsed_ms);
                ui.run_subtitle.set(format!(
                    "Processed {} iterations in one call.",
                    settings.iterations
                ));
                ui.status.set(if settings.mode == SearchMode::Parallel {
                    "Parallel run completed.".to_string()
                } else {
                    "Sequential run completed.".to_string()
                });
            }
            Err(err) => {
                ui.status
                    .set(format!("Error: failed to decode result: {err}"));
            }
        },
        Err(err) => {
            ui.status.set(format!("Error: {}", js_error_message(err)));
        }
    }

    set_running_view(&ui, settings.mode, false);
}

fn set_running_view(ui: &UiState, mode: SearchMode, running: bool) {
    ui.is_running.set(running);
    ui.run_mode.set(mode);

    if running {
        ui.run_started_at_ms.set(Date::now());
        ui.run_subtitle.set(
            "Evaluating tours with 2-opt local search. Larger instances can take longer."
                .to_string(),
        );
        ui.run_elapsed.set("Elapsed: 0.0 s".to_string());

        ui.run_ticker.borrow_mut().take();

        let run_started_at_ms = ui.run_started_at_ms;
        let run_elapsed = ui.run_elapsed;
        let ticker = Interval::new(200, move || {
            let secs = (Date::now() - run_started_at_ms.get_untracked()) / 1000.0;
            run_elapsed.set(format!("Elapsed: {secs:.1} s"));
        });

        *ui.run_ticker.borrow_mut() = Some(ticker);
        return;
    }

    ui.run_ticker.borrow_mut().take();
}

async fn allow_running_overlay_to_render() {
    TimeoutFuture::new(48).await;
}

fn clear_best(ui: &UiState) {
    ui.best_so_far.set(None);
    ui.best_distance.set("-".to_string());
    ui.elapsed.set("-".to_string());
    ui.ips.set("-".to_string());
}

fn update_stats(ui: &UiState, iterations: usize, best_distance: f64, elapsed_ms: f64) {
    ui.best_distance.set(format!("{:.3}", best_distance));
    ui.elapsed.set(format!("{:.1} ms", elapsed_ms));
    let ips = iterations as f64 / f64::max(elapsed_ms / 1000.0, 1e-9);
    ui.ips.set(format!("{ips:.0}"));
}

fn parse_u32_input(value: String, fallback: u32) -> u32 {
    value.parse::<u32>().unwrap_or(fallback)
}

fn parse_u64_input(value: String, fallback: u64) -> u64 {
    value.parse::<u64>().unwrap_or(fallback)
}

fn run_label(mode: SearchMode) -> &'static str {
    match mode {
        SearchMode::Parallel => "Running parallel search...",
        SearchMode::Sequential => "Running sequential search...",
    }
}

fn read_css_color(variable_name: &str) -> String {
    // This is a demo; if the expected CSS variable is missing, fail loudly.
    fn read(variable_name: &str) -> Option<String> {
        let window = web_sys::window()?;
        let document = window.document()?;
        let element = document.document_element()?;
        let style = window.get_computed_style(&element).ok()??;
        let value = style.get_property_value(variable_name).ok()?;
        Some(value.trim().to_string())
    }

    read(variable_name).expect("expected CSS variable to exist")
}

fn js_error_message(err: JsValue) -> String {
    err.as_string().unwrap_or_else(|| format!("{err:?}"))
}
