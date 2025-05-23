pub mod colorbox;
pub mod layout;
pub mod slider_1d;
pub mod slider_2d;

use crate::color::{Color, ColorFormat, Illuminant, RgbWorkingSpace};

use egui::{
    InnerResponse, Rect, Sense, Shape, Stroke, Ui, Vec2, Visuals, ecolor,
    style::{Selection, Widgets},
};

pub const SPACE: f32 = 7.;
pub const DOUBLE_SPACE: f32 = SPACE * 2.;
pub const HALF_SPACE: f32 = SPACE / 2.;

pub mod icon {
    #![allow(dead_code)]
    pub static ADD: &str = "\u{2795}";
    pub static COPY: &str = "\u{1F3F7}";
    pub static ZOOM_PICKER: &str = "\u{1F489}";
    pub static SETTINGS: &str = "\u{2699}";
    pub static EXPAND: &str = "\u{2B0C}";
    pub static EXPORT: &str = "\u{1F5B9}";
    pub static CLEAR: &str = "\u{1F5D1}";
    pub static DELETE: &str = "\u{1F5D9}";
    pub static PLAY: &str = "\u{25B6}";
    pub static DARK_MODE: &str = "\u{1F319}";
    pub static LIGHT_MODE: &str = "\u{2600}";
    pub static HELP: &str = "\u{FF1F}";
    pub static EDIT: &str = "\u{270F}";
    pub static APPLY: &str = "\u{2714}";
}

#[allow(dead_code)]
pub mod colors {
    use egui::{Color32, Rgba};
    use std::sync::LazyLock;

    pub static D_BG_00: LazyLock<Color32> = LazyLock::new(|| Color32::from_rgb(0x11, 0x16, 0x1b));
    pub static D_BG_0: LazyLock<Color32> = LazyLock::new(|| Color32::from_rgb(0x16, 0x1c, 0x23));
    pub static D_BG_1: LazyLock<Color32> = LazyLock::new(|| Color32::from_rgb(0x23, 0x2d, 0x38));
    pub static D_BG_2: LazyLock<Color32> = LazyLock::new(|| Color32::from_rgb(0x31, 0x3f, 0x4e));
    pub static D_BG_3: LazyLock<Color32> = LazyLock::new(|| Color32::from_rgb(0x41, 0x53, 0x67));
    pub static D_FG_0: LazyLock<Color32> = LazyLock::new(|| Color32::from_rgb(0xe5, 0xde, 0xd6));
    pub static D_BG_00_TRANSPARENT: LazyLock<Color32> =
        LazyLock::new(|| Rgba::from(*D_BG_00).multiply(0.96).into());
    pub static D_BG_0_TRANSPARENT: LazyLock<Color32> =
        LazyLock::new(|| Rgba::from(*D_BG_0).multiply(0.96).into());
    pub static D_BG_1_TRANSPARENT: LazyLock<Color32> =
        LazyLock::new(|| Rgba::from(*D_BG_1).multiply(0.96).into());
    pub static D_BG_2_TRANSPARENT: LazyLock<Color32> =
        LazyLock::new(|| Rgba::from(*D_BG_2).multiply(0.96).into());
    pub static D_BG_3_TRANSPARENT: LazyLock<Color32> =
        LazyLock::new(|| Rgba::from(*D_BG_3).multiply(0.96).into());
    pub static L_BG_0: LazyLock<Color32> = LazyLock::new(|| Color32::from_rgb(0xbf, 0xbf, 0xbf));
    pub static L_BG_1: LazyLock<Color32> = LazyLock::new(|| Color32::from_rgb(0xd4, 0xd3, 0xd4));
    pub static L_BG_2: LazyLock<Color32> = LazyLock::new(|| Color32::from_rgb(0xd9, 0xd9, 0xd9));
    pub static L_BG_3: LazyLock<Color32> = LazyLock::new(|| Color32::from_rgb(0xea, 0xea, 0xea));
    pub static L_BG_4: LazyLock<Color32> = LazyLock::new(|| Color32::from_rgb(0xf9, 0xf9, 0xf9));
    pub static L_BG_5: LazyLock<Color32> = LazyLock::new(|| Color32::from_rgb(0xff, 0xff, 0xff));
    pub static L_BG_0_TRANSPARENT: LazyLock<Color32> =
        LazyLock::new(|| Rgba::from(*L_BG_0).multiply(0.86).into());
    pub static L_BG_1_TRANSPARENT: LazyLock<Color32> =
        LazyLock::new(|| Rgba::from(*L_BG_1).multiply(0.86).into());
    pub static L_BG_2_TRANSPARENT: LazyLock<Color32> =
        LazyLock::new(|| Rgba::from(*L_BG_2).multiply(0.86).into());
    pub static L_BG_3_TRANSPARENT: LazyLock<Color32> =
        LazyLock::new(|| Rgba::from(*L_BG_3).multiply(0.86).into());
    pub static L_BG_4_TRANSPARENT: LazyLock<Color32> =
        LazyLock::new(|| Rgba::from(*L_BG_4).multiply(0.86).into());
    pub static L_BG_5_TRANSPARENT: LazyLock<Color32> =
        LazyLock::new(|| Rgba::from(*L_BG_5).multiply(0.86).into());
    pub static L_FG_0: LazyLock<Color32> = LazyLock::new(|| *D_BG_0);
}
use colors::*;
use egui::epaint::Shadow;

pub fn drop_target<R>(
    ui: &mut Ui,
    can_accept_what_is_being_dragged: bool,
    body: impl FnOnce(&mut Ui) -> R,
) -> InnerResponse<R> {
    let is_being_dragged = ui.ctx().dragged_id().is_some();

    let margin = Vec2::splat(4.0);

    let outer_rect_bounds = ui.available_rect_before_wrap();
    let inner_rect = outer_rect_bounds.shrink2(margin);
    let where_to_put_background = ui.painter().add(Shape::Noop);
    let mut content_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(inner_rect)
            .layout(*ui.layout()),
    );
    let ret = body(&mut content_ui);
    let outer_rect = Rect::from_min_max(outer_rect_bounds.min, content_ui.min_rect().max + margin);
    let (rect, response) = ui.allocate_at_least(outer_rect.size(), Sense::hover());

    let style = if is_being_dragged && can_accept_what_is_being_dragged && response.hovered() {
        ui.visuals().widgets.active
    } else {
        ui.visuals().widgets.inactive
    };

    let mut fill = style.bg_fill;
    let mut stroke = style.bg_stroke;
    if is_being_dragged && !can_accept_what_is_being_dragged {
        // gray out:
        fill = ecolor::tint_color_towards(fill, ui.visuals().window_fill());
        stroke.color = ecolor::tint_color_towards(stroke.color, ui.visuals().window_fill());
    }

    ui.painter().set(
        where_to_put_background,
        egui::epaint::RectShape {
            corner_radius: style.corner_radius,
            fill,
            stroke,
            rect,
            blur_width: 0.0,
            stroke_kind: egui::StrokeKind::Outside,
            round_to_pixels: None,
            brush: None,
        },
    );

    InnerResponse::new(ret, response)
}

pub fn color_tooltip(
    color: &Color,
    display_format: ColorFormat,
    ws: RgbWorkingSpace,
    illuminant: Illuminant,
    text: Option<&str>,
) -> String {
    format!(
        "{}\n\n{}",
        color.display(display_format, ws, illuminant),
        text.unwrap_or_default()
    )
}

pub fn dark_visuals() -> Visuals {
    let mut widgets = Widgets::dark();
    widgets.noninteractive.bg_fill = *D_BG_2_TRANSPARENT;
    widgets.inactive.bg_fill = *D_BG_1_TRANSPARENT;
    widgets.hovered.bg_fill = *D_BG_2_TRANSPARENT;
    widgets.active.bg_fill = *D_BG_3_TRANSPARENT;

    Visuals {
        dark_mode: true,
        override_text_color: Some(*D_FG_0),
        selection: Selection {
            bg_fill: *D_BG_3_TRANSPARENT,
            stroke: Stroke::new(0.7, *D_FG_0),
        },
        popup_shadow: Shadow::NONE,
        widgets,
        ..Default::default()
    }
}
