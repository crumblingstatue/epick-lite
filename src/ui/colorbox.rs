use crate::{color::Color, context::FrameCtx, render::render_color, ui::color_tooltip};

use egui::Ui;

pub const COLORBOX_PICK_TOOLTIP: &str =
    "Primary click: set current\nMiddle click: save color\nSecondary click: copy color";
pub const COLORBOX_DRAG_TOOLTIP: &str = "Drag and drop to change the order of colors";

#[derive(Clone)]
pub struct ColorBox {
    size: egui::Vec2,
    border: bool,
    color: Color,
    label: bool,
    hover_help: Option<String>,
}

impl ColorBox {
    pub fn builder() -> ColorBoxBuilder {
        ColorBoxBuilder::default()
    }

    pub fn size(&self) -> egui::Vec2 {
        self.size
    }

    pub fn border(&self) -> bool {
        self.border
    }

    pub fn color(&self) -> Color {
        self.color
    }

    pub fn label(&self) -> bool {
        self.label
    }

    pub fn hover_help(&self) -> Option<&str> {
        self.hover_help.as_deref()
    }

    pub fn display(&self, ctx: &mut FrameCtx<'_>, ui: &mut Ui) -> Option<egui::Response> {
        let color = self.color();
        let display_str = ctx.app.display_color(&color);
        let format = ctx.app.display_format();
        let on_hover = color_tooltip(
            &color,
            format,
            ctx.app.settings.rgb_working_space,
            ctx.app.settings.illuminant,
            self.hover_help(),
        );
        let tex_allocator = &mut ctx.tex_allocator();
        let resp = render_color(
            ui,
            tex_allocator,
            ctx.tex_manager,
            self.color().color32(),
            self.size(),
            Some(&on_hover),
            self.border(),
        );
        if let Some(resp) = resp {
            if self.label() {
                ui.add(
                    egui::Label::new(egui::RichText::new(display_str).monospace())
                        .wrap_mode(egui::TextWrapMode::Extend),
                );
            }

            if resp.clicked() {
                ctx.app.picker.set_cur_color(color);
            }

            if resp.middle_clicked() {
                ctx.app.add_color(color);
            }

            if resp.secondary_clicked() {
                ui.ctx().copy_text(ctx.app.clipboard_color(&color));
            }
            return Some(resp);
        }
        None
    }
}

#[derive(Clone)]
pub struct ColorBoxBuilder {
    size: egui::Vec2,
    border: bool,
    color: Color,
    label: bool,
    hover_help: Option<String>,
}

impl Default for ColorBoxBuilder {
    fn default() -> Self {
        Self {
            size: (0., 0.).into(),
            border: false,
            color: Color::white(),
            label: false,
            hover_help: None,
        }
    }
}

impl ColorBoxBuilder {
    pub fn build(self) -> ColorBox {
        ColorBox {
            size: self.size,
            border: self.border,
            color: self.color,
            label: self.label,
            hover_help: self.hover_help,
        }
    }

    pub fn size(mut self, size: impl Into<egui::Vec2>) -> Self {
        self.size = size.into();
        self
    }

    pub fn border(mut self, border: bool) -> Self {
        self.border = border;
        self
    }

    pub fn label(mut self, label: bool) -> Self {
        self.label = label;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn hover_help(mut self, hover_help: impl Into<String>) -> Self {
        self.hover_help = Some(hover_help.into());
        self
    }
}
