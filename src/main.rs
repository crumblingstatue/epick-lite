mod app;
mod color;
mod color_picker;
mod context;
mod display_picker;
mod error;
mod keybinding;
mod math;
mod render;
mod screen_size;
mod settings;
mod ui;
mod zoom_picker;

fn get_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(1)
}

fn elapsed(timestamp: u64) -> u64 {
    get_timestamp() - timestamp
}

const APP_CANVAS_ID: &str = "epick-lite - Color Picker";

fn main() {
    let opts = eframe::NativeOptions::default();
    eframe::run_native(APP_CANVAS_ID, opts, Box::new(|ctx| Ok(app::App::init(ctx)))).unwrap();
}
