use gloo_timers::{callback::Interval, future::TimeoutFuture};
use js_sys::Date;
use leptos::html;
use leptos::prelude::*;
use leptos::prelude::{
    AriaAttributes, ClassAttribute, ElementChild, GlobalAttributes, NodeRefAttribute, PropAttribute,
};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{JsFuture, spawn_local};
use wasm_bindings::RunResult;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

mod components;
mod computation;
mod locations;
mod wasm_bindings;

use components::code_card::CodeCard;

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
    status: RwSignal<String>,
    iterations: RwSignal<u32>,
    threads: RwSignal<u32>,
    chunk_size: RwSignal<u32>,
    seed: RwSignal<u64>,
    num_cities: RwSignal<u32>,
    points: RwSignal<Vec<locations::Location>>,
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
    let points = RwSignal::new(locations::create_locations(
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
    let canvas_ref = NodeRef::new();

    let ui = UiState {
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

    Effect::new({
        let canvas_ref = canvas_ref.clone();
        let points = ui.points;
        let best_so_far = ui.best_so_far;
        move |_| {
            let current_points = points.get();
            let current_best = best_so_far.get();
            draw_scene(&canvas_ref, &current_points, current_best.as_ref());
        }
    });

    spawn_local({
        let canvas_ref = canvas_ref.clone();
        let points = ui.points;
        let best_so_far = ui.best_so_far;
        let status = ui.status;
        async move {
            TimeoutFuture::new(24).await;
            let current_points = points.get_untracked();
            let current_best = best_so_far.get_untracked();
            draw_scene(&canvas_ref, &current_points, current_best.as_ref());
            status.set("Ready".to_string());
        }
    });

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

            <section class="card">
                <div
                    class="run-overlay"
                    class:active=move || ui.is_running.get()
                    aria-live="polite"
                    aria-hidden=move || (!ui.is_running.get()).to_string()
                >
                    <div class="run-overlay-card">
                        <div class="run-overlay-top">
                            <span class="spinner" aria-hidden="true"></span>
                            <p class="run-title">{move || if ui.run_mode.get() == SearchMode::Parallel {
                                "Running parallel search..."
                            } else {
                                "Running sequential search..."
                            }}</p>
                        </div>
                        <p class="run-subtitle">{move || ui.run_subtitle.get()}</p>
                        <p class="run-elapsed">{move || ui.run_elapsed.get()}</p>
                        <div class="run-bar" aria-hidden="true"></div>
                    </div>
                </div>

                <div class="control-panel">
                    <div class="controls">
                        <label>
                            Number of cities
                            <input
                                id="numCities"
                                type="number"
                                min="5"
                                max="200"
                                prop:value=move || ui.num_cities.get().to_string()
                                on:input={
                                    let ui = ui.clone();
                                    move |ev| {
                                        let next_value = parse_u32_input(input_value(&ev), ui.num_cities.get_untracked());
                                        ui.num_cities.set(next_value);
                                        ui.points.set(locations::create_locations(ui.seed.get_untracked(), next_value));
                                        clear_best(&ui);
                                        ui.status.set(format!("Updated problem size to {next_value} cities."));
                                    }
                                }
                            />
                        </label>
                        <label>
                            Iterations
                            <input
                                id="iterations"
                                type="number"
                                min="1"
                                max="200000"
                                prop:value=move || ui.iterations.get().to_string()
                                on:input={
                                    let ui = ui.clone();
                                    move |ev| {
                                        let next_value = parse_u32_input(input_value(&ev), ui.iterations.get_untracked());
                                        ui.iterations.set(next_value.clamp(1, 200_000));
                                    }
                                }
                            />
                        </label>
                        <label>
                            Threads (1..16)
                            <input
                                id="threads"
                                type="number"
                                min="1"
                                max="16"
                                prop:value=move || ui.threads.get().to_string()
                                on:input={
                                    let ui = ui.clone();
                                    move |ev| {
                                        let next_value = parse_u32_input(input_value(&ev), ui.threads.get_untracked());
                                        ui.threads.set(next_value.clamp(MIN_THREADS, MAX_THREADS));
                                    }
                                }
                            />
                        </label>
                        <label>
                            Chunk size
                            <input
                                id="chunkSize"
                                type="number"
                                min="0"
                                max="1048576"
                                prop:value=move || ui.chunk_size.get().to_string()
                                on:input={
                                    let ui = ui.clone();
                                    move |ev| {
                                        let next_value = parse_u32_input(input_value(&ev), ui.chunk_size.get_untracked());
                                        ui.chunk_size.set(next_value);
                                        ui.status.set(format!("Chunk size set to {next_value}."));
                                    }
                                }
                            />
                        </label>
                        <label>
                            Seed
                            <input
                                id="seed"
                                type="number"
                                min="1"
                                max="99999999"
                                prop:value=move || ui.seed.get().to_string()
                                on:input={
                                    let ui = ui.clone();
                                    move |ev| {
                                        let next_value = parse_u64_input(input_value(&ev), ui.seed.get_untracked());
                                        ui.seed.set(next_value);
                                        let num_cities = ui.num_cities.get_untracked();
                                        ui.points.set(locations::create_locations(next_value, num_cities));
                                        clear_best(&ui);
                                        ui.status.set(format!("Updated city seed to {next_value}."));
                                    }
                                }
                            />
                        </label>
                    </div>

                    <div class="actions">
                        <button
                            id="runParallel"
                            prop:disabled=move || ui.is_running.get()
                            on:click={
                                let ui = ui.clone();
                                move |_| {
                                    spawn_local(run_search(ui.clone(), SearchMode::Parallel));
                                }
                            }
                        >
                            Run parallel
                        </button>
                        <button
                            id="runSequential"
                            prop:disabled=move || ui.is_running.get()
                            on:click={
                                let ui = ui.clone();
                                move |_| {
                                    spawn_local(run_search(ui.clone(), SearchMode::Sequential));
                                }
                            }
                        >
                            Run sequential
                        </button>
                        <button
                            id="reset"
                            prop:disabled=move || ui.is_running.get()
                            on:click={
                                let ui = ui.clone();
                                move |_| {
                                    clear_best(&ui);
                                    let seed = ui.seed.get_untracked();
                                    let num_cities = ui.num_cities.get_untracked();
                                    ui.points.set(locations::create_locations(seed, num_cities));
                                    ui.status.set("Best tour reset. Ready for a fresh run.".to_string());
                                }
                            }
                        >
                            Reset
                        </button>
                    </div>

                    <div class="status-value" aria-live="polite">{move || ui.status.get()}</div>
                </div>

                <div class="stats">
                    <div class="stat">
                        <h3>Best Distance</h3>
                        <p>{move || ui.best_distance.get()}</p>
                    </div>
                    <div class="stat">
                        <h3>Elapsed</h3>
                        <p>{move || ui.elapsed.get()}</p>
                    </div>
                    <div class="stat">
                        <h3>Iterations/s</h3>
                        <p>{move || ui.ips.get()}</p>
                    </div>
                </div>

                <canvas node_ref=canvas_ref id="canvas" width="920" height="430"></canvas>
            </section>
        </main>
    }
}

fn clear_best(ui: &UiState) {
    ui.best_so_far.set(None);
    ui.best_distance.set("-".to_string());
    ui.elapsed.set("-".to_string());
    ui.ips.set("-".to_string());
}

async fn run_search(ui: UiState, mode: SearchMode) {
    let settings = RunSettings {
        mode,
        iterations: ui.iterations.get_untracked().max(1),
        threads: ui.threads.get_untracked().clamp(MIN_THREADS, MAX_THREADS),
        chunk_size: ui.chunk_size.get_untracked(),
        seed: ui.seed.get_untracked(),
        num_cities: ui.num_cities.get_untracked().clamp(MIN_CITIES, MAX_CITIES),
    };

    ensure_points_for_cities(&ui, settings.num_cities);

    ui.is_running.set(true);
    ui.run_mode.set(mode);
    ui.run_subtitle.set(
        "Evaluating tours with 2-opt local search. Larger instances can take longer.".to_string(),
    );
    ui.run_elapsed.set("Elapsed: 0.0 s".to_string());
    ui.status.set(
        if mode == SearchMode::Parallel {
            "Running parallel search..."
        } else {
            "Running sequential search..."
        }
        .to_string(),
    );
    ui.run_started_at_ms.set(Date::now());

    ui.run_ticker.borrow_mut().take();

    let run_elapsed = ui.run_elapsed;
    let started_at = ui.run_started_at_ms.get_untracked();
    ui.run_ticker.replace(Some(Interval::new(200, move || {
        let secs = (Date::now() - started_at) / 1000.0;
        run_elapsed.set(format!("Elapsed: {secs:.1} s"));
    })));

    TimeoutFuture::new(24).await;

    let result = match run_search_once(&settings).await {
        Ok(result) => result,
        Err(err) => {
            ui.status.set(format!("Error: {err:?}"));
            ui.is_running.set(false);
            ui.run_ticker.borrow_mut().take();
            return;
        }
    };

    if ui
        .best_so_far
        .get_untracked()
        .as_ref()
        .is_none_or(|best| result.best_distance < best.best_distance)
    {
        ui.best_so_far.set(Some(result.clone()));
    }

    ui.best_distance.set(format!("{:.3}", result.best_distance));
    ui.elapsed.set(format!("{:.1} ms", result.elapsed_ms));
    let iterations_per_second = result.iterations as f64 / (result.elapsed_ms / 1000.0).max(1e-9);
    ui.ips.set(format!("{:.0}", iterations_per_second));
    ui.run_subtitle.set(format!(
        "Processed {} iterations in one call.",
        settings.iterations
    ));
    ui.status.set(
        if mode == SearchMode::Parallel {
            "Parallel run completed."
        } else {
            "Sequential run completed."
        }
        .to_string(),
    );

    ui.is_running.set(false);
    ui.run_ticker.borrow_mut().take();
}

async fn run_search_once(settings: &RunSettings) -> Result<RunResult, JsValue> {
    let settings_js = serde_wasm_bindgen::to_value(settings)
        .map_err(|e| JsValue::from_str(&format!("failed to serialize run settings: {e}")))?;
    let result = JsFuture::from(runSearchOnce(settings_js)).await?;
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| JsValue::from_str(&format!("failed to deserialize run result: {e}")))
}

fn ensure_points_for_cities(ui: &UiState, num_cities: u32) {
    if ui.points.get_untracked().len() as u32 == num_cities {
        return;
    }

    ui.points.set(locations::create_locations(
        ui.seed.get_untracked(),
        num_cities,
    ));
    clear_best(ui);
}

fn parse_u32_input(raw: String, fallback: u32) -> u32 {
    raw.trim().parse::<u32>().unwrap_or(fallback)
}

fn parse_u64_input(raw: String, fallback: u64) -> u64 {
    raw.trim().parse::<u64>().unwrap_or(fallback)
}

fn input_value(event: &web_sys::Event) -> String {
    event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::HtmlInputElement>().ok())
        .map(|input| input.value())
        .unwrap_or_default()
}

fn draw_scene(
    canvas_ref: &NodeRef<html::Canvas>,
    points: &[locations::Location],
    best_so_far: Option<&RunResult>,
) {
    let Some(canvas) = canvas_ref.get() else {
        return;
    };

    let Some(ctx) = canvas_2d_context(&canvas) else {
        return;
    };

    draw_points(&ctx, &canvas, points);
    if let Some(best) = best_so_far {
        draw_tour(&ctx, &canvas, points, &best.best_tour);
    }
}

fn canvas_2d_context(canvas: &HtmlCanvasElement) -> Option<CanvasRenderingContext2d> {
    canvas
        .get_context("2d")
        .ok()
        .flatten()?
        .dyn_into::<CanvasRenderingContext2d>()
        .ok()
}

fn read_css_color(variable_name: &str, fallback: &str) -> String {
    let Some(window) = web_sys::window() else {
        return fallback.to_string();
    };

    let Some(document) = window.document() else {
        return fallback.to_string();
    };

    let Some(root) = document.document_element() else {
        return fallback.to_string();
    };

    match window.get_computed_style(&root) {
        Ok(Some(style)) => style
            .get_property_value(variable_name)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| fallback.to_string()),
        _ => fallback.to_string(),
    }
}

fn canvas_background_color() -> String {
    read_css_color("--code-block-bg", "#0f172a")
}

fn city_node_color() -> String {
    read_css_color("--city-node", "#f59e0b")
}

fn tour_line_color() -> String {
    read_css_color("--tour-line", "#1d4ed8")
}

fn draw_points(
    ctx: &CanvasRenderingContext2d,
    canvas: &HtmlCanvasElement,
    locations: &[locations::Location],
) {
    ctx.set_fill_style_str(&canvas_background_color());
    ctx.fill_rect(
        0.0,
        0.0,
        f64::from(canvas.width()),
        f64::from(canvas.height()),
    );

    let mapped = map_points(canvas, locations);
    ctx.set_fill_style_str(&city_node_color());

    for p in mapped {
        ctx.begin_path();
        let _ = ctx.arc(p.x, p.y, 6.0, 0.0, std::f64::consts::PI * 2.0);
        ctx.fill();
    }
}

fn draw_tour(
    ctx: &CanvasRenderingContext2d,
    canvas: &HtmlCanvasElement,
    locations: &[locations::Location],
    tour: &[usize],
) {
    draw_points(ctx, canvas, locations);
    if tour.is_empty() {
        return;
    }

    let mapped = map_points(canvas, locations);
    ctx.begin_path();
    ctx.set_stroke_style_str(&tour_line_color());
    ctx.set_line_width(2.0);

    let first = mapped[tour[0]];
    ctx.move_to(first.x, first.y);

    for idx in &tour[1..] {
        let p = mapped[*idx];
        ctx.line_to(p.x, p.y);
    }

    ctx.line_to(first.x, first.y);
    ctx.stroke();
}

fn map_points(canvas: &HtmlCanvasElement, locations: &[locations::Location]) -> Vec<Point> {
    let pad = 28.0;
    let xs: Vec<f64> = locations.iter().map(|p| p.x).collect();
    let ys: Vec<f64> = locations.iter().map(|p| p.y).collect();
    let min_x = xs.iter().copied().fold(f64::INFINITY, f64::min);
    let max_x = xs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let min_y = ys.iter().copied().fold(f64::INFINITY, f64::min);
    let max_y = ys.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let span_x = (max_x - min_x).max(1.0);
    let span_y = (max_y - min_y).max(1.0);

    locations
        .iter()
        .map(|p| Point {
            x: pad + ((p.x - min_x) / span_x) * (f64::from(canvas.width()) - pad * 2.0),
            y: f64::from(canvas.height())
                - (pad + ((p.y - min_y) / span_y) * (f64::from(canvas.height()) - pad * 2.0)),
        })
        .collect()
}

#[derive(Clone, Copy)]
struct Point {
    x: f64,
    y: f64,
}
