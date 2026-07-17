use gloo_timers::callback::Interval;
use js_sys::Date;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, HtmlInputElement, InputEvent};
use yew::TargetCast;
use yew::prelude::*;

mod computation;
mod locations;

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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
struct RunResult {
    best_tour: Vec<usize>,
    best_distance: f64,
    iterations: usize,
    elapsed_ms: f64,
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

#[wasm_bindgen(js_namespace = globalThis)]
unsafe extern "C" {
    fn runSearchOnce(settings: JsValue) -> js_sys::Promise;
    fn highlightCodeBlocks();
}

#[wasm_bindgen]
#[allow(unused_variables)]
pub fn init_parallel_runtime(num_threads: u32) -> js_sys::Promise {
    #[cfg(target_feature = "atomics")]
    return orx_parallel::init_thread_pool(num_threads as usize);

    #[cfg(not(target_feature = "atomics"))]
    panic!("init_parallel_runtime requires a wasm target with atomics and shared memory enabled")
}

#[wasm_bindgen]
pub fn locations(seed: u64, num_cities: u32) -> Result<JsValue, JsValue> {
    let locations = locations::create_locations(seed, num_cities);
    serde_wasm_bindgen::to_value(&locations)
        .map_err(|e| JsValue::from_str(&format!("failed to serialize locations: {e}")))
}

#[wasm_bindgen]
pub fn run_best_tour_par(
    iterations: u32,
    seed: u64,
    threads: u32,
    chunk_size: u32,
    num_cities: u32,
) -> Result<JsValue, JsValue> {
    let iterations = iterations.max(1) as usize;
    let threads = threads as usize;
    let chunk_size = chunk_size as usize;
    let num_cities = locations::clamp_num_cities(num_cities);
    let locations = locations::create_locations(seed, num_cities as u32);
    let started_at = Date::now();
    let output =
        computation::run_search_parallel(iterations, seed, threads, chunk_size, &locations);
    let elapsed_ms = Date::now() - started_at;
    run_output_to_js(output, elapsed_ms)
}

#[wasm_bindgen]
pub fn run_best_tour_seq(iterations: u32, seed: u64, num_cities: u32) -> Result<JsValue, JsValue> {
    let iterations = iterations.max(1) as usize;
    let num_cities = locations::clamp_num_cities(num_cities);
    let locations = locations::create_locations(seed, num_cities as u32);
    let started_at = Date::now();
    let output = computation::run_search_sequential(iterations, seed, &locations);
    let elapsed_ms = Date::now() - started_at;
    run_output_to_js(output, elapsed_ms)
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start_app() {
    console_error_panic_hook::set_once();
    yew::Renderer::<App>::new().render();
}

#[function_component(App)]
fn app() -> Html {
    let status = use_state(|| "Initializing...".to_string());
    let iterations = use_state(|| 10_000_u32);
    let threads = use_state(|| 4_u32);
    let chunk_size = use_state(|| 0_u32);
    let seed = use_state(|| 42_u64);
    let num_cities = use_state(|| 50_u32);
    let points = use_state(|| generate_points(*seed, *num_cities));
    let best_so_far = use_state(|| None::<RunResult>);
    let best_distance = use_state(|| "-".to_string());
    let elapsed = use_state(|| "-".to_string());
    let ips = use_state(|| "-".to_string());
    let is_running = use_state(|| false);
    let run_mode = use_state(|| SearchMode::Parallel);
    let run_subtitle =
        use_state(|| "Working through candidate tours. Larger runs can take a while.".to_string());
    let run_elapsed = use_state(|| "Elapsed: 0.0 s".to_string());
    let run_started_at_ms = use_state(|| 0.0_f64);
    let run_ticker = use_state(|| None::<Interval>);
    let canvas_ref = use_node_ref();

    {
        let status = status.clone();
        use_effect(move || {
            if status.as_str() == "Initializing..." {
                status.set("Ready".to_string());
            }
            || ()
        });
    }

    {
        let canvas_ref = canvas_ref.clone();
        let points = points.clone();
        let best_so_far = best_so_far.clone();
        use_effect(move || {
            let current_points = (*points).clone();
            let current_best = (*best_so_far).clone();
            draw_scene(&canvas_ref, &current_points, current_best.as_ref());
            || ()
        });
    }

    {
        use_effect(move || {
            highlightCodeBlocks();
            || ()
        });
    }

    let on_num_cities = {
        let num_cities = num_cities.clone();
        let points = points.clone();
        let seed = seed.clone();
        let best_so_far = best_so_far.clone();
        let best_distance = best_distance.clone();
        let elapsed = elapsed.clone();
        let ips = ips.clone();
        let status = status.clone();
        Callback::from(move |event: InputEvent| {
            let next_value = parse_u32_input(input_value(&event), *num_cities);
            num_cities.set(next_value);
            points.set(generate_points(*seed, next_value));
            clear_best(&best_so_far, &best_distance, &elapsed, &ips);
            status.set(format!("Updated problem size to {next_value} cities."));
        })
    };

    let on_iterations = {
        let iterations = iterations.clone();
        Callback::from(move |event: InputEvent| {
            let next_value = parse_u32_input(input_value(&event), *iterations);
            iterations.set(next_value.clamp(1, 200_000));
        })
    };

    let on_threads = {
        let threads = threads.clone();
        Callback::from(move |event: InputEvent| {
            let next_value = parse_u32_input(input_value(&event), *threads);
            threads.set(next_value.clamp(MIN_THREADS, MAX_THREADS));
        })
    };

    let on_chunk_size = {
        let chunk_size = chunk_size.clone();
        let status = status.clone();
        Callback::from(move |event: InputEvent| {
            let next_value = parse_u32_input(input_value(&event), *chunk_size);
            chunk_size.set(next_value);
            status.set(format!("Chunk size set to {next_value}."));
        })
    };

    let on_seed = {
        let seed = seed.clone();
        let num_cities = num_cities.clone();
        let points = points.clone();
        let best_so_far = best_so_far.clone();
        let best_distance = best_distance.clone();
        let elapsed = elapsed.clone();
        let ips = ips.clone();
        let status = status.clone();
        Callback::from(move |event: InputEvent| {
            let next_value = parse_u64_input(input_value(&event), *seed);
            seed.set(next_value);
            points.set(generate_points(next_value, *num_cities));
            clear_best(&best_so_far, &best_distance, &elapsed, &ips);
            status.set(format!("Updated city seed to {next_value}."));
        })
    };

    let run_parallel = {
        let iterations = iterations.clone();
        let threads = threads.clone();
        let chunk_size = chunk_size.clone();
        let seed = seed.clone();
        let num_cities = num_cities.clone();
        let points = points.clone();
        let best_so_far = best_so_far.clone();
        let best_distance = best_distance.clone();
        let elapsed = elapsed.clone();
        let ips = ips.clone();
        let is_running = is_running.clone();
        let run_mode = run_mode.clone();
        let run_subtitle = run_subtitle.clone();
        let run_elapsed = run_elapsed.clone();
        let run_started_at_ms = run_started_at_ms.clone();
        let run_ticker = run_ticker.clone();
        let status = status.clone();
        Callback::from(move |_| {
            spawn_local(run_search_async(
                SearchMode::Parallel,
                iterations.clone(),
                threads.clone(),
                chunk_size.clone(),
                seed.clone(),
                num_cities.clone(),
                points.clone(),
                best_so_far.clone(),
                best_distance.clone(),
                elapsed.clone(),
                ips.clone(),
                is_running.clone(),
                run_mode.clone(),
                run_subtitle.clone(),
                run_elapsed.clone(),
                run_started_at_ms.clone(),
                run_ticker.clone(),
                status.clone(),
            ));
        })
    };

    let run_sequential = {
        let iterations = iterations.clone();
        let threads = threads.clone();
        let chunk_size = chunk_size.clone();
        let seed = seed.clone();
        let num_cities = num_cities.clone();
        let points = points.clone();
        let best_so_far = best_so_far.clone();
        let best_distance = best_distance.clone();
        let elapsed = elapsed.clone();
        let ips = ips.clone();
        let is_running = is_running.clone();
        let run_mode = run_mode.clone();
        let run_subtitle = run_subtitle.clone();
        let run_elapsed = run_elapsed.clone();
        let run_started_at_ms = run_started_at_ms.clone();
        let run_ticker = run_ticker.clone();
        let status = status.clone();
        Callback::from(move |_| {
            spawn_local(run_search_async(
                SearchMode::Sequential,
                iterations.clone(),
                threads.clone(),
                chunk_size.clone(),
                seed.clone(),
                num_cities.clone(),
                points.clone(),
                best_so_far.clone(),
                best_distance.clone(),
                elapsed.clone(),
                ips.clone(),
                is_running.clone(),
                run_mode.clone(),
                run_subtitle.clone(),
                run_elapsed.clone(),
                run_started_at_ms.clone(),
                run_ticker.clone(),
                status.clone(),
            ));
        })
    };

    let reset = {
        let seed = seed.clone();
        let num_cities = num_cities.clone();
        let points = points.clone();
        let best_so_far = best_so_far.clone();
        let best_distance = best_distance.clone();
        let elapsed = elapsed.clone();
        let ips = ips.clone();
        let status = status.clone();
        Callback::from(move |_| {
            clear_best(&best_so_far, &best_distance, &elapsed, &ips);
            points.set(generate_points(*seed, *num_cities));
            status.set("Best tour reset. Ready for a fresh run.".to_string());
        })
    };

    let overlay_class = if *is_running {
        classes!("run-overlay", "active")
    } else {
        classes!("run-overlay")
    };

    html! {
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
                    <h1>{"Parallel computation in WASM"}</h1>
                    <p class="hero-kicker">{"A Yew TSP demo with shared-memory wasm threads using orx-parallel."}</p>
                </div>
            </header>

            <section class="intro-cards" aria-label="Example overview">
                <article class="intro-card">
                    <h2>{"Computation: Local Search for TSP"}</h2>
                    <p>
                        {"Randomly create tours and locally optimize with two-opt "}
                        <code>{"Iterations"}</code>
                        {" times, and return the best tour."}
                    </p>
                </article>
                <article class="intro-card">
                    <h2>{"Frontend"}</h2>
                    <p>{"Yew + Rust + Vite UI."}</p>
                </article>
                <article class="intro-card">
                    <CodeCard title="Sequential Code" help_title="Sequential Code Breakdown" help_body={SEQUENTIAL_HELP} code={SEQUENTIAL_CODE} />
                </article>
                <article class="intro-card">
                    <CodeCard title="Parallel Code" help_title="Parallel Code Breakdown" help_body={PARALLEL_HELP} code={PARALLEL_CODE} />
                </article>
            </section>

            <section class="card">
                <div
                    class={overlay_class}
                    aria-live="polite"
                    aria-hidden={(!*is_running).to_string()}
                >
                    <div class="run-overlay-card">
                        <div class="run-overlay-top">
                            <span class="spinner" aria-hidden="true"></span>
                            <p class="run-title">
                                {if *run_mode == SearchMode::Parallel {
                                    "Running parallel search..."
                                } else {
                                    "Running sequential search..."
                                }}
                            </p>
                        </div>
                        <p class="run-subtitle">{(*run_subtitle).clone()}</p>
                        <p class="run-elapsed">{(*run_elapsed).clone()}</p>
                        <div class="run-bar" aria-hidden="true"></div>
                    </div>
                </div>

                <div class="control-panel">
                    <div class="controls">
                        <label>
                            {"Number of cities"}
                            <input
                                id="numCities"
                                type="number"
                                min="5"
                                max="200"
                                value={(*num_cities).to_string()}
                                oninput={on_num_cities}
                            />
                        </label>
                        <label>
                            {"Iterations"}
                            <input
                                id="iterations"
                                type="number"
                                min="1"
                                max="200000"
                                value={(*iterations).to_string()}
                                oninput={on_iterations}
                            />
                        </label>
                        <label>
                            {"Threads (1..16)"}
                            <input
                                id="threads"
                                type="number"
                                min="1"
                                max="16"
                                value={(*threads).to_string()}
                                oninput={on_threads}
                            />
                        </label>
                        <label>
                            {"Chunk size"}
                            <input
                                id="chunkSize"
                                type="number"
                                min="0"
                                max="1048576"
                                value={(*chunk_size).to_string()}
                                oninput={on_chunk_size}
                            />
                        </label>
                        <label>
                            {"Seed"}
                            <input
                                id="seed"
                                type="number"
                                min="1"
                                max="99999999"
                                value={(*seed).to_string()}
                                oninput={on_seed}
                            />
                        </label>
                    </div>

                    <div class="actions">
                        <button id="runParallel" onclick={run_parallel} disabled={*is_running}>{"Run parallel"}</button>
                        <button id="runSequential" onclick={run_sequential} disabled={*is_running}>{"Run sequential"}</button>
                        <button id="reset" onclick={reset} disabled={*is_running}>{"Reset"}</button>
                    </div>

                    <div class="status-value" aria-live="polite">{(*status).clone()}</div>
                </div>

                <div class="stats">
                    <div class="stat">
                        <h3>{"Best Distance"}</h3>
                        <p>{(*best_distance).clone()}</p>
                    </div>
                    <div class="stat">
                        <h3>{"Elapsed"}</h3>
                        <p>{(*elapsed).clone()}</p>
                    </div>
                    <div class="stat">
                        <h3>{"Iterations/s"}</h3>
                        <p>{(*ips).clone()}</p>
                    </div>
                </div>

                <canvas ref={canvas_ref} id="canvas" width="920" height="430"></canvas>
            </section>
        </main>
    }
}

#[derive(Properties, PartialEq)]
struct CodeCardProps {
    title: &'static str,
    help_title: &'static str,
    help_body: &'static str,
    code: &'static str,
}

#[function_component(CodeCard)]
fn code_card(props: &CodeCardProps) -> Html {
    html! {
        <>
            <div class="code-card-header">
                <h2>{props.title}</h2>
                <details class="code-help">
                    <summary class="code-help-trigger" aria-label={format!("Show {} explanation", props.title.to_lowercase())}>
                        {"?"}
                    </summary>
                    <div class="code-help-popover" role="note">
                        <h2 class="code-help-title">{props.help_title}</h2>
                        <pre class="code-block">
                            <code class="language-rust">{props.help_body}</code>
                        </pre>
                    </div>
                </details>
            </div>
            <pre class="code-block">
                <code class="language-rust">{props.code}</code>
            </pre>
        </>
    }
}

fn run_output_to_js(
    output: computation::SearchRunOutput,
    elapsed_ms: f64,
) -> Result<JsValue, JsValue> {
    match output.best {
        Some(solution) => {
            let result = RunResult {
                best_tour: solution.tour,
                best_distance: solution.distance,
                iterations: output.iterations,
                elapsed_ms,
            };

            serde_wasm_bindgen::to_value(&result)
                .map_err(|e| JsValue::from_str(&format!("failed to serialize result: {e}")))
        }
        None => Err(JsValue::from_str(
            "no tour could be generated (unexpected empty search)",
        )),
    }
}

async fn run_search_async(
    mode: SearchMode,
    iterations: UseStateHandle<u32>,
    threads: UseStateHandle<u32>,
    chunk_size: UseStateHandle<u32>,
    seed: UseStateHandle<u64>,
    num_cities: UseStateHandle<u32>,
    points: UseStateHandle<Vec<locations::Location>>,
    best_so_far: UseStateHandle<Option<RunResult>>,
    best_distance: UseStateHandle<String>,
    elapsed: UseStateHandle<String>,
    ips: UseStateHandle<String>,
    is_running: UseStateHandle<bool>,
    run_mode: UseStateHandle<SearchMode>,
    run_subtitle: UseStateHandle<String>,
    run_elapsed: UseStateHandle<String>,
    run_started_at_ms: UseStateHandle<f64>,
    run_ticker: UseStateHandle<Option<Interval>>,
    status: UseStateHandle<String>,
) {
    let settings = RunSettings {
        mode,
        iterations: (*iterations).max(1),
        threads: (*threads).clamp(MIN_THREADS, MAX_THREADS),
        chunk_size: *chunk_size,
        seed: *seed,
        num_cities: (*num_cities).clamp(MIN_CITIES, MAX_CITIES),
    };

    ensure_points_for_cities(
        &points,
        &best_so_far,
        &best_distance,
        &elapsed,
        &ips,
        *seed,
        settings.num_cities,
    );

    is_running.set(true);
    run_mode.set(mode);
    run_subtitle.set(
        "Evaluating tours with 2-opt local search. Larger instances can take longer.".to_string(),
    );
    run_elapsed.set("Elapsed: 0.0 s".to_string());
    status.set(
        if mode == SearchMode::Parallel {
            "Running parallel search..."
        } else {
            "Running sequential search..."
        }
        .to_string(),
    );
    let started_at = Date::now();
    run_started_at_ms.set(started_at);

    run_ticker.set(None);
    let run_elapsed_state = run_elapsed.clone();
    run_ticker.set(Some(Interval::new(200, move || {
        let secs = (Date::now() - started_at) / 1000.0;
        run_elapsed_state.set(format!("Elapsed: {secs:.1} s"));
    })));

    let result = match run_search_once(&settings).await {
        Ok(result) => result,
        Err(err) => {
            status.set(format!("Error: {err:?}"));
            is_running.set(false);
            run_ticker.set(None);
            return;
        }
    };

    if best_so_far
        .as_ref()
        .is_none_or(|best| result.best_distance < best.best_distance)
    {
        best_so_far.set(Some(result.clone()));
    }

    best_distance.set(format!("{:.3}", result.best_distance));
    elapsed.set(format!("{:.1} ms", result.elapsed_ms));
    let iterations_per_second = result.iterations as f64 / (result.elapsed_ms / 1000.0).max(1e-9);
    ips.set(format!("{:.0}", iterations_per_second));
    run_subtitle.set(format!(
        "Processed {} iterations in one call.",
        settings.iterations
    ));
    status.set(
        if mode == SearchMode::Parallel {
            "Parallel run completed."
        } else {
            "Sequential run completed."
        }
        .to_string(),
    );

    is_running.set(false);
    run_ticker.set(None);
}

async fn run_search_once(settings: &RunSettings) -> Result<RunResult, JsValue> {
    let settings_js = serde_wasm_bindgen::to_value(settings)
        .map_err(|e| JsValue::from_str(&format!("failed to serialize run settings: {e}")))?;
    let result = JsFuture::from(runSearchOnce(settings_js)).await?;
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| JsValue::from_str(&format!("failed to deserialize run result: {e}")))
}

fn ensure_points_for_cities(
    points: &UseStateHandle<Vec<locations::Location>>,
    best_so_far: &UseStateHandle<Option<RunResult>>,
    best_distance: &UseStateHandle<String>,
    elapsed: &UseStateHandle<String>,
    ips: &UseStateHandle<String>,
    seed: u64,
    num_cities: u32,
) {
    if points.len() as u32 == num_cities {
        return;
    }

    points.set(generate_points(seed, num_cities));
    clear_best(best_so_far, best_distance, elapsed, ips);
}

fn clear_best(
    best_so_far: &UseStateHandle<Option<RunResult>>,
    best_distance: &UseStateHandle<String>,
    elapsed: &UseStateHandle<String>,
    ips: &UseStateHandle<String>,
) {
    best_so_far.set(None);
    best_distance.set("-".to_string());
    elapsed.set("-".to_string());
    ips.set("-".to_string());
}

fn generate_points(seed: u64, num_cities: u32) -> Vec<locations::Location> {
    let points = locations(seed, num_cities).expect("failed to generate locations");
    serde_wasm_bindgen::from_value(points).expect("failed to deserialize locations")
}

fn parse_u32_input(raw: String, fallback: u32) -> u32 {
    raw.trim().parse::<u32>().unwrap_or(fallback)
}

fn parse_u64_input(raw: String, fallback: u64) -> u64 {
    raw.trim().parse::<u64>().unwrap_or(fallback)
}

fn input_value(event: &InputEvent) -> String {
    event.target_unchecked_into::<HtmlInputElement>().value()
}

fn draw_scene(
    canvas_ref: &NodeRef,
    points: &[locations::Location],
    best_so_far: Option<&RunResult>,
) {
    let Some(canvas) = canvas_ref.cast::<HtmlCanvasElement>() else {
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
    let _ = variable_name;
    fallback.to_string()
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
    ctx.set_fill_style(&JsValue::from_str(&canvas_background_color()));
    ctx.fill_rect(
        0.0,
        0.0,
        f64::from(canvas.width()),
        f64::from(canvas.height()),
    );

    let mapped = map_points(canvas, locations);
    ctx.set_fill_style(&JsValue::from_str(&city_node_color()));

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
    ctx.set_stroke_style(&JsValue::from_str(&tour_line_color()));
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
