mod canvas_view;
mod code_card;
mod controls;
mod status;

use canvas_view::{CanvasView, draw_scene};
use code_card::CodeCard;
use computation::{Location, create_locations};
use controls::ControlsSection;
use gloo_timers::callback::Interval;
use js_sys::Date;
use serde::{Deserialize, Serialize};
use status::StatusSection;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{JsFuture, spawn_local};
use wasm_bindings::RunResult;
use web_sys::MouseEvent;
use yew::prelude::*;

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
pub(crate) enum SearchMode {
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

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchRequest {
    settings: RunSettings,
    locations: Vec<Location>,
}

#[wasm_bindgen(js_namespace = globalThis)]
unsafe extern "C" {
    fn runSearchAlgorithm(request: JsValue) -> js_sys::Promise;
    fn highlightCodeBlocks();
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start_app() {
    console_error_panic_hook::set_once();
    yew::Renderer::<App>::new().render();
}

#[function_component(App)]
pub fn app() -> Html {
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
        use_effect_with((), |_| {
            highlightCodeBlocks();
            || ()
        });
    }

    let on_num_cities_change = {
        let num_cities = num_cities.clone();
        let points = points.clone();
        let seed = seed.clone();
        let best_so_far = best_so_far.clone();
        let best_distance = best_distance.clone();
        let elapsed = elapsed.clone();
        let ips = ips.clone();
        let status = status.clone();
        Callback::from(move |next_value: u32| {
            num_cities.set(next_value);
            points.set(generate_points(*seed, next_value));
            clear_best(&best_so_far, &best_distance, &elapsed, &ips);
            status.set(format!("Updated problem size to {next_value} cities."));
        })
    };

    let on_iterations_change = {
        let iterations = iterations.clone();
        Callback::from(move |next_value: u32| {
            iterations.set(next_value.clamp(1, 200_000));
        })
    };

    let on_threads_change = {
        let threads = threads.clone();
        Callback::from(move |next_value: u32| {
            threads.set(next_value.clamp(MIN_THREADS, MAX_THREADS));
        })
    };

    let on_chunk_size_change = {
        let chunk_size = chunk_size.clone();
        let status = status.clone();
        Callback::from(move |next_value: u32| {
            chunk_size.set(next_value);
            status.set(format!("Chunk size set to {next_value}."));
        })
    };

    let on_seed_change = {
        let seed = seed.clone();
        let num_cities = num_cities.clone();
        let points = points.clone();
        let best_so_far = best_so_far.clone();
        let best_distance = best_distance.clone();
        let elapsed = elapsed.clone();
        let ips = ips.clone();
        let status = status.clone();
        Callback::from(move |next_value: u64| {
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
        Callback::from(move |_event: MouseEvent| {
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
        Callback::from(move |_event: MouseEvent| {
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

    let on_reset = {
        let seed = seed.clone();
        let num_cities = num_cities.clone();
        let points = points.clone();
        let best_so_far = best_so_far.clone();
        let best_distance = best_distance.clone();
        let elapsed = elapsed.clone();
        let ips = ips.clone();
        let status = status.clone();
        Callback::from(move |_event: MouseEvent| {
            clear_best(&best_so_far, &best_distance, &elapsed, &ips);
            points.set(generate_points(*seed, *num_cities));
            status.set("Best tour reset. Ready for a fresh run.".to_string());
        })
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

            <ControlsSection
                iterations={*iterations}
                threads={*threads}
                chunk_size={*chunk_size}
                seed={*seed}
                num_cities={*num_cities}
                is_running={*is_running}
                status={(*status).clone()}
                on_num_cities_change={on_num_cities_change}
                on_iterations_change={on_iterations_change}
                on_threads_change={on_threads_change}
                on_chunk_size_change={on_chunk_size_change}
                on_seed_change={on_seed_change}
                on_run_parallel={run_parallel}
                on_run_sequential={run_sequential}
                on_reset={on_reset}
            />

            <section class="card">
                <StatusSection
                    is_running={*is_running}
                    run_mode={*run_mode}
                    run_subtitle={(*run_subtitle).clone()}
                    run_elapsed={(*run_elapsed).clone()}
                    best_distance={(*best_distance).clone()}
                    elapsed={(*elapsed).clone()}
                    ips={(*ips).clone()}
                />
                <CanvasView canvas_ref={canvas_ref.clone()} />
            </section>
        </main>
    }
}

async fn run_search_async(
    mode: SearchMode,
    iterations: UseStateHandle<u32>,
    threads: UseStateHandle<u32>,
    chunk_size: UseStateHandle<u32>,
    seed: UseStateHandle<u64>,
    num_cities: UseStateHandle<u32>,
    points: UseStateHandle<Vec<Location>>,
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

    let request = SearchRequest {
        settings: settings.clone(),
        locations: (*points).clone(),
    };

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

    let result = match run_search_once(&request).await {
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

async fn run_search_once(request: &SearchRequest) -> Result<RunResult, JsValue> {
    let request_js = serde_wasm_bindgen::to_value(request)
        .map_err(|e| JsValue::from_str(&format!("failed to serialize run request: {e}")))?;
    let result = JsFuture::from(runSearchAlgorithm(request_js)).await?;
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| JsValue::from_str(&format!("failed to deserialize run result: {e}")))
}

fn ensure_points_for_cities(
    points: &UseStateHandle<Vec<Location>>,
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

fn generate_points(seed: u64, num_cities: u32) -> Vec<Location> {
    create_locations(seed, num_cities)
}
