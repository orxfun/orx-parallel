use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct StatusSectionProps {
    pub is_running: bool,
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
                        <p class="run-title">{"Running search..."}</p>
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
