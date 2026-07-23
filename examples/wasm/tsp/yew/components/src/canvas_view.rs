use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct CanvasViewProps {
    pub canvas_ref: NodeRef,
}

#[function_component(CanvasView)]
pub fn canvas_view(props: &CanvasViewProps) -> Html {
    html! {
        <canvas ref={props.canvas_ref.clone()} id="canvas" width="920" height="430"></canvas>
    }
}
