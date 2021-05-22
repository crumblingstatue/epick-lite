use crate::app::render::{tex_color, TextureManager};
use crate::app::{color_tooltip, SavedColors};
use crate::color::{
    analogous, color_as_hex, complementary, create_shades, create_tints, split_complementary,
    tetradic, triadic,
};
use crate::save_to_clipboard;

use egui::{color::Color32, ComboBox, Vec2};
use egui::{vec2, ScrollArea, Slider, Ui};
use std::convert::AsRef;

#[derive(Debug, PartialEq)]
pub enum SchemeType {
    Complementary,
    Triadic,
    Tetradic,
    Analogous,
    SplitComplementary,
}

impl AsRef<str> for SchemeType {
    fn as_ref(&self) -> &str {
        match &self {
            SchemeType::Complementary => "complementary",
            SchemeType::Triadic => "triadic",
            SchemeType::Tetradic => "tetradic",
            SchemeType::Analogous => "analogous",
            SchemeType::SplitComplementary => "split complementary",
        }
    }
}

pub struct SchemeGenerator {
    pub numof_shades: u8,
    pub numof_tints: u8,
    pub shade_color_size: f32,
    pub tint_color_size: f32,
    pub scheme_color_size: f32,
    pub base_color: Option<Color32>,
    pub tex_mngr: TextureManager,
    pub scheme_ty: SchemeType,
}

impl Default for SchemeGenerator {
    fn default() -> Self {
        Self {
            numof_shades: 6,
            numof_tints: 6,
            shade_color_size: 100.,
            tint_color_size: 100.,
            scheme_color_size: 200.,
            base_color: None,
            tex_mngr: TextureManager::default(),
            scheme_ty: SchemeType::Complementary,
        }
    }
}

impl SchemeGenerator {
    pub fn set_cur_color(&mut self, color: Color32) {
        self.base_color = Some(color);
    }

    fn color_box_label_under(
        &mut self,
        color: &Color32,
        size: Vec2,
        ui: &mut Ui,
        tex_allocator: &mut Option<&mut dyn epi::TextureAllocator>,
        saved_colors: &mut SavedColors,
    ) {
        ui.vertical(|ui| {
            self._color_box(color, size, ui, tex_allocator, saved_colors, true);
        });
    }

    fn color_box_label_side(
        &mut self,
        color: &Color32,
        size: Vec2,
        ui: &mut Ui,
        tex_allocator: &mut Option<&mut dyn epi::TextureAllocator>,
        saved_colors: &mut SavedColors,
    ) {
        ui.horizontal(|ui| {
            self._color_box(color, size, ui, tex_allocator, saved_colors, true);
        });
    }

    #[allow(dead_code)]
    fn color_box_no_label(
        &mut self,
        color: &Color32,
        size: Vec2,
        ui: &mut Ui,
        tex_allocator: &mut Option<&mut dyn epi::TextureAllocator>,
        saved_colors: &mut SavedColors,
    ) {
        self._color_box(color, size, ui, tex_allocator, saved_colors, false);
    }

    fn _color_box(
        &mut self,
        color: &Color32,
        size: Vec2,
        ui: &mut Ui,
        tex_allocator: &mut Option<&mut dyn epi::TextureAllocator>,
        saved_colors: &mut SavedColors,
        with_label: bool,
    ) {
        let hex = color_as_hex(&color);
        let color_box = tex_color(
            ui,
            tex_allocator,
            &mut self.tex_mngr,
            color.clone(),
            size,
            Some(&color_tooltip(&color)),
        );
        if let Some(color_box) = color_box {
            if with_label {
                ui.monospace(format!("#{}", hex));
            }

            if color_box.clicked() {
                self.set_cur_color(color.clone());
            }

            if color_box.middle_clicked() {
                saved_colors.add(color.clone());
            }

            if color_box.secondary_clicked() {
                let _ = save_to_clipboard(hex);
            }
        }
    }

    pub fn tints(
        &mut self,
        ui: &mut Ui,
        tex_allocator: &mut Option<&mut dyn epi::TextureAllocator>,
        saved_colors: &mut SavedColors,
    ) {
        ui.vertical(|ui| {
            ui.heading("Tints");
            if let Some(color) = self.base_color {
                let tints = create_tints(&color, self.numof_tints);
                ui.add(Slider::new(&mut self.numof_tints, u8::MIN..=25).text("# of tints"));
                ui.add(Slider::new(&mut self.tint_color_size, 20.0..=200.).text("color size"));

                let size = vec2(self.tint_color_size, self.tint_color_size);
                ScrollArea::auto_sized()
                    .id_source("tints scroll")
                    .show(ui, |ui| {
                        tints.iter().for_each(|tint| {
                            self.color_box_label_side(tint, size, ui, tex_allocator, saved_colors);
                        });
                    });
            }
        });
    }
    pub fn shades(
        &mut self,
        ui: &mut Ui,
        tex_allocator: &mut Option<&mut dyn epi::TextureAllocator>,
        saved_colors: &mut SavedColors,
    ) {
        ui.vertical(|ui| {
            ui.heading("Shades");
            if let Some(color) = self.base_color {
                let shades = create_shades(&color, self.numof_shades);
                ui.add(Slider::new(&mut self.numof_shades, u8::MIN..=25).text("# of shades"));
                ui.add(Slider::new(&mut self.shade_color_size, 20.0..=200.).text("color size"));

                let size = vec2(self.shade_color_size, self.shade_color_size);
                ScrollArea::auto_sized()
                    .id_source("shades scroll")
                    .show(ui, |ui| {
                        shades.iter().for_each(|shade| {
                            self.color_box_label_side(shade, size, ui, tex_allocator, saved_colors);
                        });
                    });
            }
        });
    }

    pub fn schemes(
        &mut self,
        ui: &mut Ui,
        tex_allocator: &mut Option<&mut dyn epi::TextureAllocator>,
        saved_colors: &mut SavedColors,
    ) {
        ui.heading("Schemes");
        let size = vec2(self.scheme_color_size, self.scheme_color_size);

        macro_rules! cb {
            ($color:ident, $ui:ident) => {
                $ui.scope(|mut ui| {
                    self.color_box_label_under(&$color, size, &mut ui, tex_allocator, saved_colors);
                });
            };
        }

        if let Some(color) = self.base_color {
            ComboBox::from_label("Choose a type")
                .selected_text(self.scheme_ty.as_ref())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.scheme_ty,
                        SchemeType::Complementary,
                        "Complementary",
                    );
                    ui.selectable_value(&mut self.scheme_ty, SchemeType::Triadic, "Triadic");
                    ui.selectable_value(&mut self.scheme_ty, SchemeType::Tetradic, "Tetradic");
                    ui.selectable_value(&mut self.scheme_ty, SchemeType::Analogous, "Analogous");
                    ui.selectable_value(
                        &mut self.scheme_ty,
                        SchemeType::SplitComplementary,
                        "Split complementary",
                    );
                });
            ui.add(Slider::new(&mut self.scheme_color_size, 100.0..=250.).text("color size"));
            match self.scheme_ty {
                SchemeType::Complementary => {
                    let compl = complementary(&color);
                    ui.vertical(|ui| {
                        cb!(color, ui);
                        cb!(compl, ui);
                    });
                }
                SchemeType::Triadic => {
                    let tri = triadic(&color);
                    ui.vertical(|ui| {
                        let c1 = tri.0;
                        let c2 = tri.1;
                        cb!(color, ui);
                        cb!(c1, ui);
                        cb!(c2, ui);
                    });
                }
                SchemeType::Tetradic => {
                    let tetr = tetradic(&color);
                    ui.vertical(|ui| {
                        let c1 = &tetr.0;
                        let c2 = &tetr.1;
                        let c3 = &tetr.2;
                        cb!(color, ui);
                        cb!(c1, ui);
                        cb!(c2, ui);
                        cb!(c3, ui);
                    });
                }
                SchemeType::Analogous => {
                    let an = analogous(&color);
                    ui.vertical(|ui| {
                        let c1 = an.0;
                        let c2 = an.1;
                        cb!(color, ui);
                        cb!(c1, ui);
                        cb!(c2, ui);
                    });
                }
                SchemeType::SplitComplementary => {
                    let sc = split_complementary(&color);
                    ui.vertical(|ui| {
                        let c1 = sc.0;
                        let c2 = sc.1;
                        cb!(color, ui);
                        cb!(c1, ui);
                        cb!(c2, ui);
                    });
                }
            }
        }
    }

    pub fn ui(
        &mut self,
        ui: &mut Ui,
        tex_allocator: &mut Option<&mut dyn epi::TextureAllocator>,
        saved_colors: &mut SavedColors,
    ) {
        if self.base_color.is_none() {
            ui.heading("Select a color from saved colors to continue");
        } else {
            ui.columns(3, |columns| {
                self.shades(&mut columns[0], tex_allocator, saved_colors);
                self.tints(&mut columns[1], tex_allocator, saved_colors);
                self.schemes(&mut columns[2], tex_allocator, saved_colors);
            });
        }
    }
}
