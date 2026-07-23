use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use super::{
    MAX_CITIES, MAX_THREADS, MIN_CITIES, MIN_THREADS, SearchMode, UiState, clear_best,
    create_locations, parse_u32_input, parse_u64_input, run_search,
};

#[component]
pub fn ControlsSection(ui: UiState) -> impl IntoView {
    view! {
        <section class="card">
            <div class="control-panel">
                <div class="controls">
                    <label>
                        Number of cities
                        <input
                            id="numCities"
                            type="number"
                            min=MIN_CITIES.to_string()
                            max=MAX_CITIES.to_string()
                            prop:value=move || ui.num_cities.get().to_string()
                            on:input={
                                let ui = ui.clone();
                                move |ev| {
                                    let next_value = parse_u32_input(event_target_value(&ev), ui.num_cities.get_untracked());
                                    ui.num_cities.set(next_value);
                                    ui.points.set(create_locations(ui.seed.get_untracked(), next_value));
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
                                    let next_value = parse_u32_input(event_target_value(&ev), ui.iterations.get_untracked());
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
                                    let next_value = parse_u32_input(event_target_value(&ev), ui.threads.get_untracked());
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
                                    let next_value = parse_u32_input(event_target_value(&ev), ui.chunk_size.get_untracked());
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
                                    let next_value = parse_u64_input(event_target_value(&ev), ui.seed.get_untracked());
                                    ui.seed.set(next_value);
                                    let num_cities = ui.num_cities.get_untracked();
                                    ui.points.set(create_locations(next_value, num_cities));
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
                                ui.points.set(create_locations(ui.seed.get_untracked(), ui.num_cities.get_untracked()));
                                ui.status.set("Best tour reset. Ready for a fresh run.".to_string());
                            }
                        }
                    >
                        Reset
                    </button>
                </div>
            </div>

            <div id="status" class="status-value" aria-live="polite">{move || ui.status.get()}</div>
        </section>
    }
}
