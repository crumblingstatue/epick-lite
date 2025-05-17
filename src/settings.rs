use crate::{
    color::{
        ChromaticAdaptationMethod, ColorFormat, ColorHarmony, CustomPaletteFormat, Illuminant,
        PaletteFormat, RgbWorkingSpace,
    },
    ui::layout::HarmonyLayout,
};

use anyhow::{Context, Result};
use eframe::Storage;
use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

pub const DEFAULT_PIXELS_PER_POINT: f32 = 1.0;

pub fn load_global(_storage: Option<&dyn eframe::Storage>) -> Option<Settings> {
    if let Some(config_dir) = Settings::dir("epick") {
        let path = config_dir.join(Settings::FILE_NAME);

        if let Ok(settings) = Settings::load(path) {
            return Some(settings);
        }
    }

    None
}

pub fn save_global(settings: &Settings, _storage: &mut dyn Storage) {
    if let Some(dir) = Settings::dir("epick") {
        if !dir.exists() {
            let _ = std::fs::create_dir_all(&dir);
        }
        let _ = settings.save(dir.join(Settings::FILE_NAME));
    }
}

fn enabled() -> bool {
    true
}

fn is_false(it: &bool) -> bool {
    !*it
}

fn is_true(it: &bool) -> bool {
    *it
}

fn is_default_harmony_layout(it: &HarmonyLayout) -> bool {
    *it == HarmonyLayout::default()
}

fn is_default_harmony(it: &ColorHarmony) -> bool {
    *it == ColorHarmony::default()
}

fn is_default_color_size(it: &f32) -> bool {
    *it == DEFAULT_COLOR_SIZE
}

const DEFAULT_COLOR_SIZE: f32 = 100.;

fn default_color_size() -> f32 {
    DEFAULT_COLOR_SIZE
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Settings {
    #[serde(default)]
    pub color_display_format: ColorDisplayFmtEnum,
    #[serde(default)]
    pub color_clipboard_format: Option<ColorDisplayFmtEnum>,
    #[serde(default)]
    pub palette_clipboard_format: PaletteFormat,
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub saved_color_formats: HashMap<String, String>,
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub saved_palette_formats: HashMap<String, CustomPaletteFormat>,
    #[serde(default)]
    pub rgb_working_space: RgbWorkingSpace,
    #[serde(default)]
    pub chromatic_adaptation_method: ChromaticAdaptationMethod,
    #[serde(default)]
    pub illuminant: Illuminant,
    #[serde(default = "enabled")]
    #[serde(skip_serializing_if = "is_true")]
    pub cache_colors: bool,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default_harmony")]
    pub harmony: ColorHarmony,
    #[serde(default = "enabled")]
    #[serde(skip_serializing_if = "is_true")]
    pub is_dark_mode: bool,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_default_harmony_layout")]
    pub harmony_layout: HarmonyLayout,
    #[serde(default = "default_color_size")]
    #[serde(skip_serializing_if = "is_default_color_size")]
    pub harmony_color_size: f32,
    #[serde(default)]
    #[serde(skip_serializing_if = "is_false")]
    pub harmony_display_color_label: bool,
    #[serde(default = "enabled")]
    #[serde(skip_serializing_if = "is_true")]
    pub harmony_display_box: bool,
    /// Automatically copy the picked color to the clipboard
    #[serde(default)]
    #[serde(skip_serializing_if = "is_false")]
    pub auto_copy_picked_color: bool,
    #[serde(default = "default_pixels_per_point")]
    #[serde(skip_serializing_if = "is_default_pixels_per_point")]
    pub pixels_per_point: f32,
}

fn default_pixels_per_point() -> f32 {
    DEFAULT_PIXELS_PER_POINT
}

fn is_default_pixels_per_point(ppp: &f32) -> bool {
    *ppp == DEFAULT_PIXELS_PER_POINT
}

impl Default for Settings {
    fn default() -> Self {
        let ws = RgbWorkingSpace::default();
        Self {
            color_display_format: ColorDisplayFmtEnum::default(),
            color_clipboard_format: None,
            palette_clipboard_format: PaletteFormat::default(),
            saved_color_formats: HashMap::default(),
            saved_palette_formats: HashMap::default(),
            rgb_working_space: ws,
            chromatic_adaptation_method: ChromaticAdaptationMethod::default(),
            illuminant: ws.reference_illuminant(),
            cache_colors: true,
            is_dark_mode: true,
            harmony: ColorHarmony::default(),
            harmony_layout: HarmonyLayout::default(),
            harmony_color_size: DEFAULT_COLOR_SIZE,
            harmony_display_color_label: false,
            harmony_display_box: true,
            auto_copy_picked_color: false,
            pixels_per_point: DEFAULT_PIXELS_PER_POINT,
        }
    }
}

impl Settings {
    pub const FILE_NAME: &'static str = "settings.ron";

    /// Loads the settings from the configuration file located at `path`. The configuration file is
    /// expected to be a valid ron file.
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let data = fs::read_to_string(path).context("failed to read configuration file")?;
        ron::from_str(&data).context("Failed to parse configuration file")
    }

    /// Saves this settings as ron file in the provided `path`.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let mut data = String::with_capacity(128);
        ron::ser::to_writer_pretty(&mut data, self, PrettyConfig::default())?;
        fs::write(path, &data).context("failed to write settings to file")
    }

    /// Returns system directory where configuration should be placed joined by the `name` parameter.
    pub fn dir(name: impl AsRef<str>) -> Option<PathBuf> {
        let name = name.as_ref();
        if let Some(dir) = dirs::config_dir() {
            return Some(dir.join(name));
        }

        if let Some(dir) = dirs::home_dir() {
            return Some(dir.join(name));
        }

        None
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Default)]
pub enum ColorDisplayFmtEnum {
    #[serde(rename = "hex")]
    #[default]
    Hex,
    #[serde(rename = "hex-uppercase")]
    HexUppercase,
    #[serde(rename = "css-rgb")]
    CssRgb,
    #[serde(rename = "css-hsl")]
    CssHsl,
    #[serde(rename = "custom")]
    Custom(String),
}

impl AsRef<str> for ColorDisplayFmtEnum {
    fn as_ref(&self) -> &str {
        use ColorDisplayFmtEnum::*;
        match &self {
            Hex => "hex",
            HexUppercase => "hex uppercase",
            CssRgb => "css rgb",
            CssHsl => "css hsl",
            Custom(name) => name,
        }
    }
}

impl ColorDisplayFmtEnum {
    pub fn default_display_format() -> ColorFormat<'static> {
        ColorFormat::Hex
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        color::{ChromaticAdaptationMethod, ColorHarmony, Illuminant, RgbWorkingSpace},
        math::eq_f32,
        settings::{DEFAULT_COLOR_SIZE, Settings},
        ui::layout::HarmonyLayout,
    };
    use std::fs;

    #[test]
    fn loads_settings() {
        let tmp = tempfile::TempDir::new().unwrap();
        let settings_str = r#"(
    color_display_format: hex,
    color_clipboard_format: None,
    palette_clipboard_format: HexList,
    rgb_working_space: Adobe,
    chromatic_adaptation_method: VonKries,
    illuminant: D50,
    harmony_layout: gradient,
)"#;
        let path = tmp.path().join("settings.ron");
        fs::write(&path, settings_str).unwrap();

        let settings = Settings::load(&path).unwrap();
        assert_eq!(settings.illuminant, Illuminant::D50);
        assert_eq!(settings.rgb_working_space, RgbWorkingSpace::Adobe);
        assert_eq!(
            settings.chromatic_adaptation_method,
            ChromaticAdaptationMethod::VonKries
        );
        assert!(settings.cache_colors);

        assert_eq!(settings.harmony, ColorHarmony::default());
        assert_eq!(settings.harmony_layout, HarmonyLayout::Gradient);
        assert!(eq_f32(settings.harmony_color_size, DEFAULT_COLOR_SIZE));
        assert!(!settings.harmony_display_color_label);

        let path = tmp.path().join("new_settings.ron");
        settings.save(&path).unwrap();

        pretty_assertions::assert_eq!(fs::read_to_string(&path).unwrap(), settings_str);
    }
}
