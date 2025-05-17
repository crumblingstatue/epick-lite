use crate::{
    app::{CentralPanelTab, DARK_VISUALS},
    color::{Color, ColorFormat, Palettes},
    color_picker::ColorPicker,
    error::append_global_error,
    render::{TextureAllocator, TextureManager},
    screen_size::ScreenSize,
    settings,
    settings::{ColorDisplayFmtEnum, Settings},
};

use eframe::{CreationContext, Storage};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppCtx {
    pub settings: Settings,

    pub picker: ColorPicker,

    pub palettes: Palettes,
    pub palettes_tab_color_size: f32,
    pub palettes_tab_display_label: bool,

    pub screen_size: ScreenSize,
    /// Color under cursor
    pub cursor_pick_color: Color,
    pub central_panel_tab: CentralPanelTab,

    pub show_zoom_window: bool,
}

impl Default for AppCtx {
    fn default() -> Self {
        Self {
            settings: Settings::default(),

            picker: ColorPicker::default(),

            palettes: Palettes::default(),
            palettes_tab_color_size: 50.,
            palettes_tab_display_label: false,

            screen_size: ScreenSize::Desktop(0., 0.),
            cursor_pick_color: Color::black(),
            central_panel_tab: CentralPanelTab::Picker,

            show_zoom_window: false,
        }
    }
}
impl AppCtx {
    /// Initialize a new context
    pub fn new(context: &CreationContext) -> Self {
        Self {
            settings: settings::load_global(context.storage).unwrap_or_default(),

            picker: ColorPicker::default(),

            palettes: Palettes::default(),
            palettes_tab_color_size: 50.,
            palettes_tab_display_label: false,

            screen_size: ScreenSize::Desktop(0., 0.),
            cursor_pick_color: Color::black(),
            central_panel_tab: CentralPanelTab::Picker,

            show_zoom_window: false,
        }
    }

    /// Current color display format
    pub fn display_format(&self) -> ColorFormat {
        match self.settings.color_display_format {
            ColorDisplayFmtEnum::Hex => ColorFormat::Hex,
            ColorDisplayFmtEnum::HexUppercase => ColorFormat::HexUpercase,
            ColorDisplayFmtEnum::CssRgb => ColorFormat::CssRgb,
            ColorDisplayFmtEnum::CssHsl => ColorFormat::CssHsl {
                degree_symbol: true,
            },
            ColorDisplayFmtEnum::Custom(ref name) => {
                if self.settings.saved_color_formats.contains_key(name) {
                    ColorFormat::Custom(&self.settings.saved_color_formats[name])
                } else {
                    append_global_error(format!("Custom color format `{name}` not found"));
                    ColorDisplayFmtEnum::default_display_format()
                }
            }
        }
    }

    /// Format a color as a string using display color format from settings
    pub fn display_color(&self, color: &Color) -> String {
        color.display(
            self.display_format(),
            self.settings.rgb_working_space,
            self.settings.illuminant,
        )
    }

    /// Format a color as a string using clipboard color format from settings
    pub fn clipboard_color(&self, color: &Color) -> String {
        let format = match self
            .settings
            .color_clipboard_format
            .as_ref()
            .unwrap_or(&self.settings.color_display_format)
        {
            ColorDisplayFmtEnum::Hex => ColorFormat::Hex,
            ColorDisplayFmtEnum::HexUppercase => ColorFormat::HexUpercase,
            ColorDisplayFmtEnum::CssRgb => ColorFormat::CssRgb,
            ColorDisplayFmtEnum::CssHsl => ColorFormat::CssHsl {
                degree_symbol: false,
            },
            ColorDisplayFmtEnum::Custom(name) => {
                if self.settings.saved_color_formats.contains_key(name) {
                    ColorFormat::Custom(&self.settings.saved_color_formats[name])
                } else {
                    append_global_error(format!("Custom color format `{name}` not found"));
                    ColorDisplayFmtEnum::default_display_format()
                }
            }
        };
        color.display(
            format,
            self.settings.rgb_working_space,
            self.settings.illuminant,
        )
    }

    /// Load palettes from appropriate location based on the target arch
    pub fn load_palettes(&mut self, _storage: Option<&dyn Storage>) {
        if self.settings.cache_colors
            && let Some(path) = Palettes::dir("epick")
        {
            match Palettes::load(path.join(Palettes::FILE_NAME)) {
                Ok(palettes) => self.palettes = palettes,
                Err(e) => append_global_error(format!("failed to load palettes, {e:?}")),
            }
        }
    }

    /// Save palettes to appropriate location based on the target arch
    pub fn save_palettes(&self, _storage: &mut dyn Storage) {
        if let Some(dir) = Palettes::dir("epick") {
            if !dir.exists() {
                let _ = std::fs::create_dir_all(&dir);
            }
            if let Err(e) = self.palettes.save(dir.join(Palettes::FILE_NAME)) {
                append_global_error(format!("failed to save palettes, {e:?}"));
            }
        }
    }

    /// Adds a color to the currently selected palette
    pub fn add_color(&mut self, color: Color) {
        if !self.palettes.current_mut().palette.add(color) {
            let color_str = self.display_color(&color);
            append_global_error(format!("Color {color_str} already saved!"));
        }
    }

    pub fn add_cur_color(&mut self) {
        self.add_color(self.picker.current_color)
    }

    pub fn check_settings_change(&mut self) {
        if self.settings.chromatic_adaptation_method
            != self.picker.sliders.chromatic_adaptation_method
        {
            self.picker.sliders.chromatic_adaptation_method =
                self.settings.chromatic_adaptation_method;
        }
        if self.settings.rgb_working_space != self.picker.sliders.rgb_working_space {
            self.picker.new_workspace = Some(self.settings.rgb_working_space);
            if self.settings.illuminant != self.picker.sliders.illuminant {
                self.picker.new_illuminant = Some(self.settings.illuminant);
            }
        }
    }
}

pub struct FrameCtx<'frame> {
    pub app: &'frame mut AppCtx,
    pub egui: &'frame egui::Context,
    pub tex_manager: &'frame mut TextureManager,
    pub frame: Option<&'frame mut eframe::Frame>,
}

impl FrameCtx<'_> {
    pub fn tex_allocator(&self) -> TextureAllocator {
        Some(self.egui.tex_manager())
    }

    pub fn set_dark_theme(&mut self) {
        self.app.settings.is_dark_mode = true;
        self.egui.set_visuals(DARK_VISUALS.clone());
    }

    pub fn set_styles(&mut self, screen_size: ScreenSize) {
        self.app.screen_size = screen_size;

        let slider_size = match screen_size {
            ScreenSize::Phone(w, _) => w * 0.5,
            ScreenSize::Desktop(w, _) if w > 1500. => w * 0.2,
            ScreenSize::Tablet(w, _) | ScreenSize::Laptop(w, _) | ScreenSize::Desktop(w, _) => {
                w * 0.35
            }
        };

        let mut style = (*self.egui.style()).clone();
        style.spacing.slider_width = slider_size / 2.;
        self.egui.set_style(style);
    }

    pub fn set_window_size(&mut self, _size: egui::Vec2) {
        if let Some(_frame) = &mut self.frame {
            // TODO: Fix for egui 0.26
            //frame.set_window_size(size);
        }
    }
}
