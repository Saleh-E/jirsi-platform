//! Signature Pad - HTML5 Canvas drawing component
//!
//! Features:
//! - Mouse and touch drawing
//! - Clear button
//! - Export to base64 PNG
//! - Customizable pen color and width

use leptos::*;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::{MouseEvent, TouchEvent};

#[component]
pub fn SignaturePad(
    /// Callback when signature is saved (base64 PNG)
    on_save: Callback<String>,
    /// Canvas width
    #[prop(default = 600)] width: u32,
    /// Canvas height
    #[prop(default = 200)] height: u32,
    /// Pen color
    #[prop(default = "#000000".to_string())] pen_color: String,
    /// Pen width
    #[prop(default = 2.0)] pen_width: f64,
) -> impl IntoView {
    let canvas_ref = create_node_ref::<html::Canvas>();
    let (is_drawing, set_is_drawing) = create_signal(false);
    let (last_x, set_last_x) = create_signal(0.0);
    let (last_y, set_last_y) = create_signal(0.0);
    
    let get_context = move || {
        canvas_ref.get().and_then(|canvas| {
            canvas.get_context("2d")
                .ok()
                .flatten()
                .and_then(|ctx| ctx.dyn_into::<web_sys::CanvasRenderingContext2d>().ok())
        })
    };
    
    let start_drawing = move |x: f64, y: f64| {
        set_is_drawing.set(true);
        set_last_x.set(x);
        set_last_y.set(y);
    };
    
    let draw = move |x: f64, y: f64| {
        if !is_drawing.get() {
            return;
        }
        
        if let Some(ctx) = get_context() {
            ctx.begin_path();
            ctx.set_stroke_style_str(&pen_color);
            ctx.set_line_width(pen_width);
            ctx.set_line_cap("round");
            ctx.move_to(last_x.get(), last_y.get());
            ctx.line_to(x, y);
            ctx.stroke();
            
            set_last_x.set(x);
            set_last_y.set(y);
        }
    };
    
    let stop_drawing = move || {
        set_is_drawing.set(false);
    };
    
    let clear = move |_| {
        if let Some(ctx) = get_context() {
            ctx.clear_rect(0.0, 0.0, width as f64, height as f64);
        }
    };
    
    let save = move |_| {
        if let Some(canvas) = canvas_ref.get() {
            if let Ok(data_url) = canvas.to_data_url() {
                on_save.call(data_url);
            }
        }
    };
    
    let draw_rc = Rc::new(draw);
    let draw_clone1 = draw_rc.clone();
    let draw_clone2 = draw_rc.clone();
    
    let on_mouse_down = move |ev: MouseEvent| {
        if let Some(canvas) = canvas_ref.get() {
            let rect = canvas.get_bounding_client_rect();
            start_drawing(ev.client_x() as f64 - rect.left(), ev.client_y() as f64 - rect.top());
        }
    };
    
    let on_mouse_move = move |ev: MouseEvent| {
        if let Some(canvas) = canvas_ref.get() {
            let rect = canvas.get_bounding_client_rect();
            draw_clone1(ev.client_x() as f64 - rect.left(), ev.client_y() as f64 - rect.top());
        }
    };
    
    let on_touch_start = move |ev: TouchEvent| {
        ev.prevent_default();
        if let Some(touch) = ev.touches().get(0) {
            if let Some(canvas) = canvas_ref.get() {
                let rect = canvas.get_bounding_client_rect();
                start_drawing(touch.client_x() as f64 - rect.left(), touch.client_y() as f64 - rect.top());
            }
        }
    };
    
    let on_touch_move = move |ev: TouchEvent| {
        ev.prevent_default();
        if let Some(touch) = ev.touches().get(0) {
            if let Some(canvas) = canvas_ref.get() {
                let rect = canvas.get_bounding_client_rect();
            draw_clone2(touch.client_x() as f64 - rect.left(), touch.client_y() as f64 - rect.top());
            }
        }
    };

    view! {
        <div class="signature-pad bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-700 rounded-lg p-4">
            <canvas
                node_ref=canvas_ref
                width=width
                height=height
                class="border border-gray-200 dark:border-gray-600 rounded cursor-crosshair bg-white"
                on:mousedown=on_mouse_down
                on:mousemove=on_mouse_move
                on:mouseup=move |_| stop_drawing()
                on:mouseleave=move |_| stop_drawing()
                on:touchstart=on_touch_start
                on:touchmove=on_touch_move
                on:touchend=move |_| stop_drawing()
            ></canvas>
            
            <div class="flex gap-2 mt-3">
                <button
                    type="button"
                    class="px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded border border-gray-300 dark:border-gray-600"
                    on:click=clear
                >
                    "Clear"
                </button>
                <button
                    type="button"
                    class="px-4 py-2 bg-blue-600 text-white hover:bg-blue-700 rounded"
                    on:click=save
                >
                    "Save Signature"
                </button>
            </div>
        </div>
    }
}
