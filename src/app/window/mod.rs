mod custom_formats;
mod export;
mod help;
mod palette_formats;
mod settings;

use crate::ui::colors::*;
use egui::CornerRadius;

pub use custom_formats::CustomFormatsWindow;
use egui::{Frame, Margin, Slider, Stroke, Ui, epaint::Shadow};
pub use export::ExportWindow;
pub use help::HelpWindow;
pub use palette_formats::PaletteFormatsWindow;
pub use settings::SettingsWindow;

pub const WINDOW_X_OFFSET: f32 = 10.;
pub const WINDOW_Y_OFFSET: f32 = 30.;

pub fn default_frame(is_dark_mode: bool) -> Frame {
    Frame {
        fill: if is_dark_mode {
            *D_BG_1_TRANSPARENT
        } else {
            *L_BG_3_TRANSPARENT
        },
        inner_margin: Margin::symmetric(15, 15),
        corner_radius: CornerRadius::same(5),
        shadow: if is_dark_mode {
            Shadow {
                offset: [2, 2],
                blur: 40,
                spread: 3,
                color: egui::Color32::BLACK,
            }
        } else {
            Shadow {
                offset: [2, 2],
                blur: 40,
                spread: 3,
                color: egui::Color32::GRAY,
            }
        },
        stroke: if is_dark_mode {
            Stroke::new(2., *D_BG_00)
        } else {
            Stroke::new(2., *L_BG_2)
        },
        ..Default::default()
    }
}

pub fn apply_default_style(ui: &mut Ui, is_dark_mode: bool) {
    let widgets = &mut ui.style_mut().visuals.widgets;
    if is_dark_mode {
        widgets.inactive.bg_fill = *D_BG_2_TRANSPARENT;
    } else {
        widgets.inactive.bg_fill = *L_BG_2_TRANSPARENT;
    }
}

#[derive(Debug)]
pub struct ShadesWindow {
    pub num_of_shades: u8,
    pub shade_color_size: f32,
}

impl Default for ShadesWindow {
    fn default() -> Self {
        Self {
            num_of_shades: 6,
            shade_color_size: DEFAULT_COLOR_SIZE,
        }
    }
}

impl ShadesWindow {
    pub fn sliders(&mut self, ui: &mut Ui) {
        ui.add(
            Slider::new(&mut self.num_of_shades, u8::MIN..=50)
                .clamping(egui::SliderClamping::Always)
                .text("# of shades"),
        );
        ui.add(
            Slider::new(&mut self.shade_color_size, 20.0..=200.)
                .clamping(egui::SliderClamping::Always)
                .text("color size"),
        );
    }
}

#[derive(Debug)]
pub struct TintsWindow {
    pub num_of_tints: u8,
    pub tint_color_size: f32,
}

impl Default for TintsWindow {
    fn default() -> Self {
        Self {
            num_of_tints: 6,
            tint_color_size: DEFAULT_COLOR_SIZE,
        }
    }
}

impl TintsWindow {
    pub fn sliders(&mut self, ui: &mut Ui) {
        ui.add(
            Slider::new(&mut self.num_of_tints, u8::MIN..=50)
                .clamping(egui::SliderClamping::Always)
                .text("# of tints"),
        );
        ui.add(
            Slider::new(&mut self.tint_color_size, 20.0..=200.)
                .clamping(egui::SliderClamping::Always)
                .text("color size"),
        );
    }
}

#[derive(Debug)]
pub struct HuesWindow {
    pub num_of_hues: u8,
    pub hue_color_size: f32,
    pub hues_step: f32,
}

const DEFAULT_COLOR_SIZE: f32 = 48.0;

impl Default for HuesWindow {
    fn default() -> Self {
        Self {
            num_of_hues: 4,
            hue_color_size: DEFAULT_COLOR_SIZE,
            hues_step: 0.05,
        }
    }
}

impl HuesWindow {
    pub fn sliders(&mut self, ui: &mut Ui) {
        ui.add(
            Slider::new(&mut self.hues_step, 0.01..=0.1)
                .clamping(egui::SliderClamping::Always)
                .text("step"),
        );
        let max_hues = (0.5 / self.hues_step).round() as u8;
        if self.num_of_hues > max_hues {
            self.num_of_hues = max_hues;
        }
        ui.add(
            Slider::new(&mut self.num_of_hues, u8::MIN..=max_hues)
                .clamping(egui::SliderClamping::Always)
                .text("# of hues"),
        );
        ui.add(
            Slider::new(&mut self.hue_color_size, 20.0..=200.)
                .clamping(egui::SliderClamping::Always)
                .text("color size"),
        );
    }
}
