use computation::Location;
use wasm_bindgen::JsCast;
use wasm_bindings::RunResult;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};
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

pub(crate) fn draw_scene(
    canvas_ref: &NodeRef,
    points: &[Location],
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

fn read_css_color(variable_name: &str) -> String {
    // This is a demo; if the expected CSS variable is missing, fail loudly.
    fn read(variable_name: &str) -> Option<String> {
        let window = web_sys::window()?;
        let document = window.document()?;
        let element = document.document_element()?;
        let style = window.get_computed_style(&element).ok()??;
        let value = style.get_property_value(variable_name).ok()?;
        Some(value.trim().to_string())
    }

    read(variable_name).expect("expected CSS variable to exist")
}

fn draw_points(ctx: &CanvasRenderingContext2d, canvas: &HtmlCanvasElement, locations: &[Location]) {
    ctx.set_fill_style_str(&read_css_color("--code-block-bg"));
    ctx.fill_rect(
        0.0,
        0.0,
        f64::from(canvas.width()),
        f64::from(canvas.height()),
    );

    let mapped = map_points(canvas, locations);
    ctx.set_fill_style_str(&read_css_color("--city-node"));

    for p in mapped {
        ctx.begin_path();
        let _ = ctx.arc(p.x, p.y, 6.0, 0.0, std::f64::consts::PI * 2.0);
        ctx.fill();
    }
}

fn draw_tour(
    ctx: &CanvasRenderingContext2d,
    canvas: &HtmlCanvasElement,
    locations: &[Location],
    tour: &[usize],
) {
    draw_points(ctx, canvas, locations);
    if tour.is_empty() {
        return;
    }

    let mapped = map_points(canvas, locations);
    ctx.begin_path();
    ctx.set_stroke_style_str(&read_css_color("--tour-line"));
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

fn map_points(canvas: &HtmlCanvasElement, locations: &[Location]) -> Vec<Point> {
    let (min_x, max_x, min_y, max_y) = locations.iter().fold(
        (
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
        ),
        |(min_x, max_x, min_y, max_y), p| {
            (
                min_x.min(p.x),
                max_x.max(p.x),
                min_y.min(p.y),
                max_y.max(p.y),
            )
        },
    );

    let span_x = (max_x - min_x).max(1.0);
    let span_y = (max_y - min_y).max(1.0);

    let pad = 28.0;

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
