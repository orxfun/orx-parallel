use crate::SearchMode;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct StatusSectionProps {
    pub is_running: bool,
    pub run_mode: SearchMode,
    pub run_subtitle: String,
    pub run_elapsed: String,
    pub best_distance: String,
    pub elapsed: String,
    pub ips: String,
}

#[function_component(StatusSection)]
pub fn status_section(props: &StatusSectionProps) -> Html {
    let overlay_class = if props.is_running {
        classes!("run-overlay", "active")
    } else {
        classes!("run-overlay")
    };

    let run_title = if props.run_mode == SearchMode::Parallel {
        "Running parallel search..."
    } else {
        "Running sequential search..."
    };

    html! {
        <>
            <div
                class={overlay_class}
                aria-live="polite"
                aria-hidden={(!props.is_running).to_string()}
            >
                <div class="run-overlay-card">
                    <div class="run-overlay-top">
                        <span class="spinner" aria-hidden="true"></span>
                        <p class="run-title">{run_title}</p>
                    </div>
                    <p class="run-subtitle">{props.run_subtitle.clone()}</p>
                    <p class="run-elapsed">{props.run_elapsed.clone()}</p>
                    <div class="run-bar" aria-hidden="true"></div>
                </div>
            </div>

            <div class="stats">
                <div class="stat">
                    <h3>{"Best Distance"}</h3>
                    <p>{props.best_distance.clone()}</p>
                </div>
                <div class="stat">
                    <h3>{"Elapsed"}</h3>
                    <p>{props.elapsed.clone()}</p>
                </div>
                <div class="stat">
                    <h3>{"Iterations/s"}</h3>
                    <p>{props.ips.clone()}</p>
                </div>
            </div>
        </>
    }
}
