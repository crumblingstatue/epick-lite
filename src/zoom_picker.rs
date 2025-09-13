#![allow(unused_imports)]
use crate::{
    app::CURRENT_COLOR_BOX_SIZE,
    context::FrameCtx,
    display_picker::{self, X11DisplayPicker},
    error::append_global_error,
    ui::{
        colorbox::{COLORBOX_PICK_TOOLTIP, ColorBox},
        icon,
    },
};

use egui::{Button, CursorIcon, Ui};
use std::rc::Rc;

use x11rb::protocol::xproto;

const ZOOM_IMAGE_WIDTH: u16 = ZOOM_WIN_WIDTH / ZOOM_SCALE as u16;
const ZOOM_IMAGE_HEIGHT: u16 = ZOOM_WIN_HEIGHT / ZOOM_SCALE as u16;
const ZOOM_SCALE: f32 = 10.;
const ZOOM_WIN_WIDTH: u16 = 160;
const ZOOM_WIN_HEIGHT: u16 = 160;
const ZOOM_IMAGE_X_OFFSET: i32 = ((ZOOM_WIN_WIDTH / 2) as f32 / ZOOM_SCALE) as i32;
const ZOOM_IMAGE_Y_OFFSET: i32 = ((ZOOM_WIN_HEIGHT / 2) as f32 / ZOOM_SCALE) as i32;

pub struct ZoomPicker {
    pub display_picker: Option<Rc<X11DisplayPicker>>,
}

impl Default for ZoomPicker {
    fn default() -> Self {
        Self {
            display_picker: crate::display_picker::init_display_picker(),
        }
    }
}
impl ZoomPicker {
    pub fn display(&mut self, ctx: &mut FrameCtx<'_>, ui: &mut Ui) {
        if let Some(picker) = self.display_picker.clone()
            && let Ok(color) = picker.get_color_under_cursor()
        {
            ctx.app.cursor_pick_color = color;
            ui.horizontal(|ui| {
                let cb = ColorBox::builder()
                    .size((CURRENT_COLOR_BOX_SIZE, CURRENT_COLOR_BOX_SIZE))
                    .color(color)
                    .label(true)
                    .hover_help(COLORBOX_PICK_TOOLTIP)
                    .border(true)
                    .build();
                cb.display(ctx, ui);
                ui.label("At cursor");
                self.zoom_picker_impl(ctx, ui, picker);
            });
        };
    }

    fn handle_zoom_picker(&mut self, ui: &mut Ui, picker: Rc<X11DisplayPicker>) {
        use egui::{Color32, ColorImage, ImageSource, TextureOptions};

        let cursor_pos = picker.get_cursor_pos().unwrap_or_default();
        if let Ok(img) = picker.get_image(
            picker.screen().root,
            (cursor_pos.0 - ZOOM_IMAGE_X_OFFSET) as i16,
            (cursor_pos.1 - ZOOM_IMAGE_Y_OFFSET) as i16,
            ZOOM_IMAGE_WIDTH,
            ZOOM_IMAGE_HEIGHT,
        ) {
            let image = egui::ColorImage {
                size: [img.width() as usize, img.height() as usize],
                source_size: egui::vec2(f32::from(img.width()), f32::from(img.height())),
                pixels: img
                    .data()
                    .chunks(4)
                    .map(|pixfmt| {
                        let [b, g, r, a] = pixfmt.try_into().unwrap();
                        Color32::from_rgba_unmultiplied(r, g, b, a)
                    })
                    .collect(),
            };
            let tex_handle = ui
                .ctx()
                .load_texture("screen-image", image, TextureOptions::NEAREST);
            let re = ui.image(egui::load::SizedTexture::new(
                tex_handle.id(),
                egui::vec2((img.width() * 10) as f32, (img.height() * 10) as f32),
            ));
            let painter = ui.painter_at(re.rect);
            painter.circle(
                re.rect.center() + egui::vec2(5.0, 5.0),
                5.0,
                egui::Color32::TRANSPARENT,
                egui::Stroke::new(1.0, egui::Color32::WHITE),
            );
        }
        if let Err(e) = picker.flush() {
            append_global_error(e);
        }
    }

    fn zoom_picker_impl(
        &mut self,
        ctx: &mut FrameCtx<'_>,
        ui: &mut Ui,
        picker: Rc<X11DisplayPicker>,
    ) {
        let re = ui.checkbox(&mut ctx.app.show_zoom_window, "Zoom window");

        if ctx.app.show_zoom_window {
            let rect = re.rect;
            let pos = egui::pos2(rect.min.x, rect.max.y);
            egui::Tooltip::always_open(
                ctx.egui.clone(),
                ui.layer_id(),
                egui::Id::new("zoomed-tooltip"),
                pos,
            )
            .show(|ui| {
                self.handle_zoom_picker(ui, picker);
            });
        }
    }
}
