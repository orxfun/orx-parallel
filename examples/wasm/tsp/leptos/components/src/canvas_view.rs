use gloo_timers::future::TimeoutFuture;
use leptos::html;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::CanvasRenderingContext2d;

use super::{Location, RunResult, UiState};

#[component]
pub fn CanvasView(
    ui: UiState,
    city_node_color: String,
    tour_line_color: String,
    canvas_background_color: String,
) -> impl IntoView {
    let canvas_ref = ui.canvas_ref.clone();
    let points = ui.points;
    let best_so_far = ui.best_so_far;
    let status = ui.status;

    Effect::new({
        let canvas_ref = canvas_ref.clone();
        let points = points;
        let best_so_far = best_so_far;
        let city_node_color = city_node_color.clone();
        let tour_line_color = tour_line_color.clone();
        let canvas_background_color = canvas_background_color.clone();
        move |_| {
            let current_points = points.get();
            let current_best = best_so_far.get();
            draw_scene(
                &canvas_ref,
                &current_points,
                current_best.as_ref(),
                &city_node_color,
                &tour_line_color,
                &canvas_background_color,
            );
        }
    });

    spawn_local({
        let canvas_ref = canvas_ref.clone();
        let points = points;
        let best_so_far = best_so_far;
        let status = status;
        let city_node_color = city_node_color.clone();
        let tour_line_color = tour_line_color.clone();
        let canvas_background_color = canvas_background_color.clone();
        async move {
            TimeoutFuture::new(24).await;
            let current_points = points.get_untracked();
            let current_best = best_so_far.get_untracked();
            draw_scene(
                &canvas_ref,
                &current_points,
                current_best.as_ref(),
                &city_node_color,
                &tour_line_color,
                &canvas_background_color,
            );
            status.set("Ready".to_string());
        }
    });

    view! {
        <canvas id="canvas" width="920" height="430" node_ref=canvas_ref></canvas>
    }
}

pub(crate) fn draw_scene(
    canvas_ref: &NodeRef<html::Canvas>,
    locations: &[Location],
    best: Option<&RunResult>,
    city_node_color: &str,
    tour_line_color: &str,
    canvas_background_color: &str,
) {
    let Some(canvas) = canvas_ref.get() else {
        return;
    };

    let Ok(Some(context)) = canvas.get_context("2d") else {
        return;
    };

    let Ok(ctx) = context.dyn_into::<CanvasRenderingContext2d>() else {
        return;
    };

    let width = canvas.width() as f64;
    let height = canvas.height() as f64;

    ctx.set_fill_style_str(canvas_background_color);
    ctx.fill_rect(0.0, 0.0, width, height);

    let mapped = map_points(locations, width, height);

    for p in &mapped {
        ctx.begin_path();
        ctx.set_fill_style_str(city_node_color);
        let _ = ctx.arc(p.0, p.1, 6.0, 0.0, std::f64::consts::PI * 2.0);
        ctx.fill();
    }

    if let Some(best) = best {
        if best.best_tour.is_empty() {
            return;
        }

        ctx.begin_path();
        ctx.set_stroke_style_str(tour_line_color);
        ctx.set_line_width(2.0);

        let first = mapped[best.best_tour[0]];
        ctx.move_to(first.0, first.1);

        for idx in best.best_tour.iter().skip(1) {
            let p = mapped[*idx];
            ctx.line_to(p.0, p.1);
        }

        ctx.line_to(first.0, first.1);
        ctx.stroke();
    }
}

fn map_points(locations: &[Location], width: f64, height: f64) -> Vec<(f64, f64)> {
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

    let span_x = f64::max(max_x - min_x, 1.0);
    let span_y = f64::max(max_y - min_y, 1.0);

    let pad = 28.0;

    locations
        .iter()
        .map(|p| {
            (
                pad + ((p.x - min_x) / span_x) * (width - pad * 2.0),
                height - (pad + ((p.y - min_y) / span_y) * (height - pad * 2.0)),
            )
        })
        .collect()
}