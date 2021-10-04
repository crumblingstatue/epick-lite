use crate::app::saved_colors::{PaletteFormat, SavedColors};

use anyhow::Result;
use egui::color::Color32;
use egui::{ComboBox, Window};
use std::path::PathBuf;
use std::{env, fs};

#[cfg(not(target_arch = "wasm32"))]
use egui::TextEdit;

#[derive(Default, Debug)]
pub struct SettingsWindow {
    pub show: bool,
    pub upper_hex: bool,
}

impl SettingsWindow {
    pub fn display(&mut self, ctx: &egui::CtxRef) {
        if self.show {
            let mut show = true;
            Window::new("settings").open(&mut show).show(ctx, |ui| {
                ui.checkbox(&mut self.upper_hex, "Show hex as uppercase");
            });

            if !show {
                self.show = false;
            }
        }
    }
}

#[derive(Debug)]
pub struct ExportWindow {
    pub show: bool,
    pub path: String,
    pub name: String,
    pub export_status: Result<String, String>,
    pub format: PaletteFormat,
    pub export_path_editable: bool,
}

impl Default for ExportWindow {
    fn default() -> Self {
        Self {
            show: false,
            format: PaletteFormat::Gimp,
            name: "".to_string(),
            export_status: Ok("".to_string()),
            path: env::current_dir()
                .map(|d| d.to_string_lossy().to_string())
                .unwrap_or_default(),
            export_path_editable: false,
        }
    }
}

impl ExportWindow {
    pub fn display(&mut self, ctx: &egui::CtxRef, saved_colors: &SavedColors) -> Result<()> {
        if self.show {
            let mut show = true;
            Window::new("export").open(&mut show).show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ComboBox::from_label("format")
                            .selected_text(self.format.as_ref())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut self.format,
                                    PaletteFormat::Gimp,
                                    PaletteFormat::Gimp.as_ref(),
                                );
                                ui.selectable_value(
                                    &mut self.format,
                                    PaletteFormat::Text,
                                    PaletteFormat::Text.as_ref(),
                                );
                            });
                    });
                    ui.label("Export path:");
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        if ui
                            .add(
                                TextEdit::singleline(&mut self.path)
                                    .enabled(self.export_path_editable),
                            )
                            .clicked()
                            && !self.export_path_editable
                        {
                            let location = if let Ok(path) = std::env::current_dir() {
                                path.to_string_lossy().to_string()
                            } else {
                                "".into()
                            };

                            match native_dialog::FileDialog::new()
                                .set_location(&location)
                                .add_filter("GIMP Palette", &["gpl"])
                                .add_filter("Text file", &["txt"])
                                .show_save_single_file()
                            {
                                Ok(Some(path)) => self.path = path.to_string_lossy().to_string(),
                                Err(_) => {
                                    self.export_path_editable = true;
                                }
                                Ok(None) => {}
                            }
                        };
                    }
                    #[cfg(target_arch = "wasm32")]
                    {
                        ui.text_edit_singleline(&mut self.path);
                    }

                    ui.label("Name:");
                    ui.text_edit_singleline(&mut self.name);

                    match &self.export_status {
                        Ok(msg) => ui.colored_label(Color32::GREEN, msg),
                        Err(msg) => ui.colored_label(Color32::RED, msg),
                    };

                    if ui.button("export").clicked() {
                        let palette = match self.format {
                            PaletteFormat::Gimp => saved_colors.as_gimp_palette(&self.name),
                            PaletteFormat::Text => saved_colors.as_text_palette(),
                        };
                        let p = PathBuf::from(&self.path);
                        let filename = format!("{}.{}", &self.name, self.format.extension());
                        if let Err(e) = fs::write(p.join(&filename), palette) {
                            self.export_status = Err(e.to_string());
                        } else {
                            self.export_status = Ok("export succesful".to_string());
                        }
                    }
                });
            });

            if !show {
                self.show = false;
            }
        }

        Ok(())
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
            shade_color_size: 100.,
        }
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
            tint_color_size: 100.,
        }
    }
}

#[derive(Debug)]
pub struct HuesWindow {
    pub num_of_hues: u8,
    pub hue_color_size: f32,
    pub hues_step: f32,
}

impl Default for HuesWindow {
    fn default() -> Self {
        Self {
            num_of_hues: 4,
            hue_color_size: 100.,
            hues_step: 0.05,
        }
    }
}