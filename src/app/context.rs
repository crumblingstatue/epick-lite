use crate::{
    app::{
        load_settings, CentralPanelTab, CursorIcon, DisplayFmtEnum, Settings, DARK_VISUALS,
        LIGHT_VISUALS,
    },
    color::{Color, DisplayFormat, Palettes},
    error::append_global_error,
    render::TextureAllocator,
    screen_size::ScreenSize,
};
use serde::{Deserialize, Serialize};

use eframe::{CreationContext, Storage};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppCtx {
    pub settings: Settings,

    pub palettes: Palettes,
    pub palettes_tab_display_label: bool,

    pub screen_size: ScreenSize,
    pub cursor_icon: CursorIcon,
    /// Color under cursor
    pub cursor_pick_color: Color,
    /// Currently selected color in the picker
    pub current_selected_color: Color,
    pub central_panel_tab: CentralPanelTab,

    pub sidepanel: SidePanelData,

    /// Is the zoom window currently dragged
    pub zoom_window_dragged: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SidePanelData {
    /// Is the side panel visible
    pub show: bool,
    /// If true palette name is currently being edited
    pub edit_palette_name: bool,
    /// When triggering palette name edit this is used to
    /// switch focus to the textedit
    pub trigger_edit_focus: bool,
    /// Width of the button toolbar on the sidebar
    pub box_width: f32,
    /// Size of the whole Sidebar response
    pub response_size: egui::Vec2,
}

impl Default for AppCtx {
    fn default() -> Self {
        Self {
            settings: Settings::default(),

            palettes: Palettes::default(),
            palettes_tab_display_label: false,

            screen_size: ScreenSize::Desktop(0., 0.),
            cursor_icon: CursorIcon::default(),
            cursor_pick_color: Color::black(),
            current_selected_color: Color::black(),
            central_panel_tab: CentralPanelTab::Picker,
            sidepanel: SidePanelData {
                show: false,
                edit_palette_name: false,
                trigger_edit_focus: false,
                box_width: 0.,
                response_size: (0., 0.).into(),
            },

            zoom_window_dragged: false,
        }
    }
}
impl AppCtx {
    pub const KEY: &'static str = "app-global-ctx";

    /// Initialize a new context
    pub fn new(context: &CreationContext) -> Self {
        Self {
            settings: load_settings(context.storage).unwrap_or_default(),

            palettes: Palettes::default(),
            palettes_tab_display_label: false,
            screen_size: ScreenSize::Desktop(0., 0.),
            cursor_icon: CursorIcon::default(),
            cursor_pick_color: Color::black(),
            current_selected_color: Color::black(),
            central_panel_tab: CentralPanelTab::Picker,
            sidepanel: SidePanelData {
                show: false,
                edit_palette_name: false,
                trigger_edit_focus: false,
                box_width: 0.,
                response_size: (0., 0.).into(),
            },

            zoom_window_dragged: false,
        }
    }

    /// Current color display format
    pub fn display_format(&self) -> DisplayFormat {
        match self.settings.color_display_format {
            DisplayFmtEnum::Hex => DisplayFormat::Hex,
            DisplayFmtEnum::HexUppercase => DisplayFormat::HexUpercase,
            DisplayFmtEnum::CssRgb => DisplayFormat::CssRgb,
            DisplayFmtEnum::CssHsl => DisplayFormat::CssHsl {
                degree_symbol: true,
            },
            DisplayFmtEnum::Custom(ref name) => {
                if self.settings.saved_color_formats.get(name).is_some() {
                    DisplayFormat::Custom(&self.settings.saved_color_formats[name])
                } else {
                    append_global_error(format!("Custom color format `{name}` not found"));
                    DisplayFmtEnum::default_display_format()
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
            DisplayFmtEnum::Hex => DisplayFormat::Hex,
            DisplayFmtEnum::HexUppercase => DisplayFormat::HexUpercase,
            DisplayFmtEnum::CssRgb => DisplayFormat::CssRgb,
            DisplayFmtEnum::CssHsl => DisplayFormat::CssHsl {
                degree_symbol: false,
            },
            DisplayFmtEnum::Custom(name) => {
                if self.settings.saved_color_formats.get(name).is_some() {
                    DisplayFormat::Custom(&self.settings.saved_color_formats[name])
                } else {
                    append_global_error(format!("Custom color format `{name}` not found"));
                    DisplayFmtEnum::default_display_format()
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
        if self.settings.cache_colors {
            #[cfg(target_arch = "wasm32")]
            if let Some(storage) = _storage {
                match Palettes::load_from_storage(storage) {
                    Ok(palettes) => self.palettes = palettes,
                    Err(e) => append_global_error(format!("failed to load palettes, {e:?}")),
                }
            }

            #[cfg(not(target_arch = "wasm32"))]
            if let Some(path) = Palettes::dir("epick") {
                match Palettes::load(path.join(Palettes::FILE_NAME)) {
                    Ok(palettes) => self.palettes = palettes,
                    Err(e) => append_global_error(format!("failed to load palettes, {e:?}")),
                }
            }
        }
    }

    /// Save palettes to appropriate location based on the target arch
    pub fn save_palettes(&self, _storage: &mut dyn Storage) {
        #[cfg(target_arch = "wasm32")]
        if self.settings.cache_colors {
            if let Err(e) = self.palettes.save_to_storage(_storage) {
                append_global_error(format!("failed to save palettes, {e:?}"));
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
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
            append_global_error(format!("Color {} already saved!", color_str));
        } else {
            self.sidepanel.show = true;
        }
    }

    /// Replaces cursor icon with `icon`
    pub fn toggle_mouse(&mut self, icon: CursorIcon) {
        self.cursor_icon = if icon == self.cursor_icon {
            CursorIcon::default()
        } else {
            icon
        }
    }
}

pub struct FrameCtx<'frame> {
    pub app: &'frame mut AppCtx,
    pub egui: &'frame egui::Context,
}

impl<'frame> FrameCtx<'frame> {
    pub fn new(app: &'frame mut AppCtx, egui: &'frame egui::Context) -> Self {
        Self { app, egui }
    }

    pub fn tex_allocator(&self) -> TextureAllocator {
        Some(self.egui.tex_manager())
    }

    pub fn is_dark_mode(&self) -> bool {
        self.app.settings.is_dark_mode
    }

    pub fn set_dark_theme(&mut self) {
        self.app.settings.is_dark_mode = true;
        self.egui.set_visuals(DARK_VISUALS.clone());
    }

    pub fn set_light_theme(&mut self) {
        self.app.settings.is_dark_mode = false;
        self.egui.set_visuals(LIGHT_VISUALS.clone());
    }

    pub fn set_theme(&mut self) {
        if self.is_dark_mode() {
            self.set_light_theme();
        } else {
            self.set_dark_theme();
        }
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
}
