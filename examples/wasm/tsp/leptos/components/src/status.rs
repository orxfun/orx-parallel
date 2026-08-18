use leptos::prelude::*;

use super::UiState;

#[component]
pub fn StatusSection(ui: UiState) -> impl IntoView {
    view! {
        <div
            id="runOverlay"
            class="run-overlay"
            class:active=move || ui.is_running.get()
            aria-live="polite"
            aria-hidden=move || (!ui.is_running.get()).to_string()
        >
            <div class="run-overlay-card">
                <div class="run-overlay-top">
                    <span class="spinner" aria-hidden="true"></span>
                    <p id="runTitle" class="run-title">"Running search..."</p>
                </div>
                <p id="runSubtitle" class="run-subtitle">{move || ui.run_subtitle.get()}</p>
                <p id="runElapsed" class="run-elapsed">{move || ui.run_elapsed.get()}</p>
                <div class="run-bar" aria-hidden="true"></div>
            </div>
        </div>

        <div class="stats">
            <div class="stat"><h3>Best Distance</h3><p id="bestDistance">{move || ui.best_distance.get()}</p></div>
            <div class="stat"><h3>Elapsed</h3><p id="elapsed">{move || ui.elapsed.get()}</p></div>
            <div class="stat"><h3>Iterations/s</h3><p id="ips">{move || ui.ips.get()}</p></div>
        </div>
    }
}
