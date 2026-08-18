use web_sys::{HtmlInputElement, InputEvent, MouseEvent};
use yew::TargetCast;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ControlsSectionProps {
    pub iterations: u32,
    pub threads: u32,
    pub chunk_size: u32,
    pub seed: u64,
    pub num_cities: u32,
    pub is_running: bool,
    pub status: String,
    pub on_num_cities_change: Callback<u32>,
    pub on_iterations_change: Callback<u32>,
    pub on_threads_change: Callback<u32>,
    pub on_chunk_size_change: Callback<u32>,
    pub on_seed_change: Callback<u64>,
    pub on_run: Callback<MouseEvent>,
    pub on_reset: Callback<MouseEvent>,
}

#[function_component(ControlsSection)]
pub fn controls_section(props: &ControlsSectionProps) -> Html {
    let on_num_cities = {
        let callback = props.on_num_cities_change.clone();
        let fallback = props.num_cities;
        Callback::from(move |event: InputEvent| {
            callback.emit(parse_u32_input(input_value(&event), fallback));
        })
    };

    let on_iterations = {
        let callback = props.on_iterations_change.clone();
        let fallback = props.iterations;
        Callback::from(move |event: InputEvent| {
            callback.emit(parse_u32_input(input_value(&event), fallback));
        })
    };

    let on_threads = {
        let callback = props.on_threads_change.clone();
        let fallback = props.threads;
        Callback::from(move |event: InputEvent| {
            callback.emit(parse_u32_input(input_value(&event), fallback));
        })
    };

    let on_chunk_size = {
        let callback = props.on_chunk_size_change.clone();
        let fallback = props.chunk_size;
        Callback::from(move |event: InputEvent| {
            callback.emit(parse_u32_input(input_value(&event), fallback));
        })
    };

    let on_seed = {
        let callback = props.on_seed_change.clone();
        let fallback = props.seed;
        Callback::from(move |event: InputEvent| {
            callback.emit(parse_u64_input(input_value(&event), fallback));
        })
    };

    html! {
        <section class="card">
            <div class="control-panel">
                <div class="controls">
                    <label>
                        {"Number of cities"}
                        <input
                            id="numCities"
                            type="number"
                            min="5"
                            max="200"
                            value={props.num_cities.to_string()}
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
                            value={props.iterations.to_string()}
                            oninput={on_iterations}
                        />
                    </label>
                    <label>
                        {"Threads (0..32)"}
                        <input
                            id="threads"
                            type="number"
                            min="0"
                            max="32"
                            value={props.threads.to_string()}
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
                            value={props.chunk_size.to_string()}
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
                            value={props.seed.to_string()}
                            oninput={on_seed}
                        />
                    </label>
                </div>

                <div class="actions">
                    <button id="run" onclick={props.on_run.clone()} disabled={props.is_running}>{"Run"}</button>
                    <button id="reset" onclick={props.on_reset.clone()} disabled={props.is_running}>{"Reset"}</button>
                </div>

                <div class="status-value" aria-live="polite">{props.status.clone()}</div>
            </div>
        </section>
    }
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
