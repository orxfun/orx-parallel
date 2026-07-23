use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct CodeCardProps {
    pub title: &'static str,
    pub help_title: &'static str,
    pub help_body: &'static str,
    pub code: &'static str,
}

#[function_component(CodeCard)]
pub fn code_card(props: &CodeCardProps) -> Html {
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
