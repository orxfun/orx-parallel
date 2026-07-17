use leptos::prelude::*;

#[component]
pub fn CodeCard(
    title: &'static str,
    help_title: &'static str,
    help_body: &'static str,
    code: &'static str,
) -> impl IntoView {
    view! {
        <>
            <div class="code-card-header">
                <h2>{title}</h2>
                <details class="code-help">
                    <summary class="code-help-trigger" aria-label=move || format!("Show {} explanation", title.to_lowercase())>
                        ?
                    </summary>
                    <div class="code-help-popover" role="note">
                        <h2 class="code-help-title">{help_title}</h2>
                        <pre class="code-block">
                            <code class="language-rust">{help_body}</code>
                        </pre>
                    </div>
                </details>
            </div>
            <pre class="code-block">
                <code class="language-rust">{code}</code>
            </pre>
        </>
    }
}
