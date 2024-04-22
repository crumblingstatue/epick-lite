#![allow(unused_imports)]
use crate::{
    app::CURRENT_COLOR_BOX_SIZE,
    context::FrameCtx,
    display_picker::{self, DisplayPickerExt},
    error::append_global_error,
    ui::{
        colorbox::{ColorBox, COLORBOX_PICK_TOOLTIP},
        icon,
    },
};

use egui::{Button, CursorIcon, Ui};
use std::rc::Rc;

#[cfg(target_os = "linux")]
use x11rb::protocol::xproto;

#[cfg(windows)]
use crate::display_picker::windows::{HWND, SW_SHOWDEFAULT, WS_BORDER, WS_POPUP};

#[cfg(target_os = "linux")]
const ZOOM_IMAGE_WIDTH: u16 = ZOOM_WIN_WIDTH / ZOOM_SCALE as u16;
#[cfg(target_os = "linux")]
const ZOOM_IMAGE_HEIGHT: u16 = ZOOM_WIN_HEIGHT / ZOOM_SCALE as u16;
#[cfg(any(target_os = "linux", windows))]
const ZOOM_SCALE: f32 = 10.;
#[cfg(any(target_os = "linux", windows))]
const ZOOM_WIN_WIDTH: u16 = 160;
#[cfg(any(target_os = "linux", windows))]
const ZOOM_WIN_HEIGHT: u16 = 160;
#[cfg(windows)]
const ZOOM_WIN_POINTER_RADIUS: u16 = ZOOM_WIN_POINTER_DIAMETER / 2;
#[cfg(any(target_os = "linux", windows))]
const ZOOM_IMAGE_X_OFFSET: i32 = ((ZOOM_WIN_WIDTH / 2) as f32 / ZOOM_SCALE) as i32;
#[cfg(any(target_os = "linux", windows))]
const ZOOM_IMAGE_Y_OFFSET: i32 = ((ZOOM_WIN_HEIGHT / 2) as f32 / ZOOM_SCALE) as i32;

pub struct ZoomPicker {
    pub display_picker: Option<Rc<dyn DisplayPickerExt>>,
    #[cfg(windows)]
    picker_window: Option<HWND>,
}

impl Default for ZoomPicker {
    fn default() -> Self {
        Self {
            display_picker: crate::display_picker::init_display_picker(),
            #[cfg(windows)]
            picker_window: None,
        }
    }
}
impl ZoomPicker {
    pub fn display(&mut self, ctx: &mut FrameCtx<'_>, ui: &mut Ui) {
        if let Some(picker) = self.display_picker.clone() {
            if let Ok(color) = picker.get_color_under_cursor() {
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
            }
        };
    }

    #[cfg(target_os = "linux")]
    fn handle_zoom_picker(&mut self, ui: &mut Ui, picker: Rc<dyn DisplayPickerExt>) {
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

    #[cfg(windows)]
    fn handle_zoom_picker(&mut self, _ui: &mut Ui, picker: Rc<dyn DisplayPickerExt>) {
        if let Some(window) = self.picker_window {
            let cursor_pos = picker.get_cursor_pos().unwrap_or_default();
            match picker.get_screenshot(
                (cursor_pos.0 - ZOOM_IMAGE_X_OFFSET),
                (cursor_pos.1 - ZOOM_IMAGE_Y_OFFSET),
                (ZOOM_WIN_WIDTH as f32 / ZOOM_SCALE) as i32,
                (ZOOM_WIN_HEIGHT as f32 / ZOOM_SCALE) as i32,
            ) {
                Ok(bitmap) => {
                    if let Err(e) = picker.render_bitmap(&bitmap, window, 0, 0, ZOOM_SCALE) {
                        append_global_error(e);
                    }
                    let left = ((ZOOM_WIN_WIDTH / 2) - ZOOM_WIN_POINTER_RADIUS) as i32;
                    let top = ((ZOOM_WIN_HEIGHT / 2) - ZOOM_WIN_POINTER_RADIUS) as i32;
                    if let Err(e) = picker.draw_rectangle(
                        window,
                        left,
                        top,
                        left + ZOOM_WIN_POINTER_DIAMETER as i32,
                        top + ZOOM_WIN_POINTER_DIAMETER as i32,
                        true,
                    ) {
                        append_global_error(e);
                    }
                }
                Err(e) => {
                    append_global_error(e);
                }
            }
            if let Err(e) = picker.move_window(
                window,
                (cursor_pos.0 + ZOOM_WIN_OFFSET),
                (cursor_pos.1 + ZOOM_WIN_OFFSET),
                ZOOM_WIN_WIDTH as i32,
                ZOOM_WIN_HEIGHT as i32,
            ) {
                append_global_error(e);
            }
        }
    }

    #[cfg(any(target_os = "linux", windows))]
    fn zoom_picker_impl(
        &mut self,
        ctx: &mut FrameCtx<'_>,
        ui: &mut Ui,
        picker: Rc<dyn DisplayPickerExt>,
    ) {
        let re = ui.checkbox(&mut ctx.app.show_zoom_window, "Zoom window");

        if ctx.app.show_zoom_window {
            let rect = re.rect;
            let pos = egui::pos2(rect.min.x, rect.max.y);
            egui::show_tooltip_at(ctx.egui, egui::Id::new("zoomed-tooltip"), Some(pos), |ui| {
                self.handle_zoom_picker(ui, picker);
            });
        }
    }

    #[cfg(not(any(target_os = "linux", windows)))]
    fn zoom_picker_impl(&mut self, _: &mut FrameCtx<'_>, _: &mut Ui, _: Rc<dyn DisplayPickerExt>) {}
}
