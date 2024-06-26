#![allow(dead_code)]
mod palette;
mod scheme;
mod sidepanel;
pub mod window;

use crate::{
    color::{Color, ColorHarmony, Gradient},
    context::{AppCtx, FrameCtx},
    error::{append_global_error, DisplayError, ERROR_STACK},
    keybinding::{default_keybindings, KeyBindings},
    render::{render_gradient, TextureManager},
    save_to_clipboard,
    screen_size::ScreenSize,
    settings::{self},
    ui::{
        colorbox::{ColorBox, COLORBOX_PICK_TOOLTIP},
        colors::*,
        dark_visuals, icon, light_visuals, DOUBLE_SPACE, HALF_SPACE, SPACE,
    },
    zoom_picker::ZoomPicker,
};
use window::{ExportWindow, HelpWindow, HuesWindow, SettingsWindow, ShadesWindow, TintsWindow};

use eframe::{CreationContext, Storage};
use egui::{
    Button, CollapsingHeader, Color32, CursorIcon, Id, Label, Layout, Margin, Rgba, RichText,
    ScrollArea, Ui, Vec2, Visuals,
};
use once_cell::sync::{Lazy, OnceCell};
use serde::{Deserialize, Serialize};
use std::sync::RwLock;

static ADD_DESCR: &str = "Add this color to saved colors";
static ERROR_DISPLAY_DURATION: u64 = 20;

pub static KEYBINDINGS: Lazy<KeyBindings> = Lazy::new(default_keybindings);
pub static LIGHT_VISUALS: Lazy<Visuals> = Lazy::new(light_visuals);
pub static DARK_VISUALS: Lazy<Visuals> = Lazy::new(dark_visuals);
pub static CONTEXT: OnceCell<RwLock<AppCtx>> = OnceCell::new();
pub static TEXTURE_MANAGER: Lazy<RwLock<TextureManager>> =
    Lazy::new(|| RwLock::new(TextureManager::default()));

pub const CURRENT_COLOR_BOX_SIZE: f32 = 40.0;

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum CentralPanelTab {
    Picker,
    Palettes,
}

#[derive(Default)]
pub struct Windows {
    pub settings: SettingsWindow,
    pub export: ExportWindow,
    pub help: HelpWindow,
    pub hues: HuesWindow,
    pub tints: TintsWindow,
    pub shades: ShadesWindow,
}

pub struct App {
    pub display_errors: Vec<DisplayError>,
    pub windows: Windows,
    pub zoom_picker: ZoomPicker,
    pub selected_slider: u8,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        if let Some(mut app_ctx) = CONTEXT.get().and_then(|ctx| ctx.write().ok()) {
            let res = TEXTURE_MANAGER.try_write();
            if let Err(e) = &res {
                append_global_error(e);
                return;
            }
            let mut tex_manager = res.unwrap();
            let mut ctx = FrameCtx {
                app: &mut app_ctx,
                egui: ctx,
                tex_manager: &mut tex_manager,
                frame: Some(frame),
            };
            ctx.egui
                .output_mut(|out| out.cursor_icon = ctx.app.cursor_icon);

            let screen_size = ScreenSize::from(ctx.egui.available_rect());
            if ctx.app.screen_size != screen_size {
                ctx.set_styles(screen_size);
            }
            ctx.egui
                .set_pixels_per_point(ctx.app.settings.pixels_per_point);

            ctx.app.check_settings_change();

            self.top_panel(&mut ctx);

            self.central_panel(&mut ctx);

            if ctx.app.sidepanel.show {
                self.side_panel(&mut ctx);
            }

            self.display_windows(&mut ctx);

            ctx.set_window_size(ctx.egui.used_size());

            ctx.app.picker.check_for_change();

            // populate display errors from the global error stack
            if let Ok(mut stack) = ERROR_STACK.try_lock() {
                while let Some(error) = stack.errors.pop_front() {
                    self.display_errors.push(error);
                }
            }

            if ctx.egui.memory(|mem| mem.focused().is_none()) {
                self.check_keys_pressed(&mut ctx);
            }

            // No need to repaint in wasm, there is no way to pick color from under the cursor anyway
            if !ctx.egui.is_pointer_over_area() {
                // This paint request makes sure that the color displayed as color under cursor
                // gets updated even when the pointer is not in the egui window area.
                ctx.egui.request_repaint();

                if ctx.app.show_zoom_window {
                    // When zooming we want to continually repaint for smooth experience
                    // even if the pointer is not over main window area
                    return;
                }

                // Otherwise sleep to save some cycles
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            ctx.app.current_selected_color = ctx.app.picker.current_color;
        }
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        if let Some(ctx) = CONTEXT.get().and_then(|ctx| ctx.read().ok()) {
            ctx.save_palettes(storage);
            settings::save_global(&ctx.settings, storage);
        }
        storage.flush();
    }
}

impl App {
    pub fn init(context: &CreationContext) -> Box<dyn eframe::App + 'static> {
        let mut app_ctx = AppCtx::new(context);

        let app = Box::new(Self {
            display_errors: Default::default(),
            windows: Windows::default(),
            zoom_picker: ZoomPicker::default(),
            selected_slider: 0,
        });

        if let Ok(mut tex_manager) = TEXTURE_MANAGER.write() {
            let mut ctx = FrameCtx {
                app: &mut app_ctx,
                egui: &context.egui_ctx,
                tex_manager: &mut tex_manager,
                frame: None,
            };

            ctx.app.load_palettes(context.storage);

            ctx.set_dark_theme();
        }

        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "Firacode".to_string(),
            egui::FontData::from_static(include_bytes!(
                "../../assets/fonts/FiraCode/FiraCode-Regular.ttf"
            )),
        );
        fonts
            .families
            .get_mut(&egui::FontFamily::Monospace)
            .unwrap()
            .insert(0, "Firacode".to_owned());

        context.egui_ctx.set_fonts(fonts);

        // TODO: Fix for egui 0.26
        /*if app_ctx.settings.pixels_per_point == DEFAULT_PIXELS_PER_POINT {
            app_ctx.settings.pixels_per_point = context
                .integration_info
                .native_pixels_per_point
                .unwrap_or(DEFAULT_PIXELS_PER_POINT);
        }*/

        CONTEXT.try_insert(RwLock::new(app_ctx)).unwrap();

        app
    }

    fn check_keys_pressed(&mut self, ctx: &mut FrameCtx) {
        for kb in KEYBINDINGS.iter() {
            if ctx.egui.input(|inp| inp.key_pressed(kb.key())) {
                let f = kb.binding();
                f(ctx)
            }
        }
    }

    fn hex_input(&self, ctx: &mut FrameCtx<'_>, ui: &mut Ui) {
        CollapsingHeader::new("Text input").show(ui, |ui| {
            ui.label("Enter a hex color: ");
            ui.horizontal(|ui| {
                let resp = ui.text_edit_singleline(&mut ctx.app.picker.hex_color);
                if (resp.lost_focus() && ui.input(|inp| inp.key_pressed(egui::Key::Enter)))
                    || ui
                        .button(icon::PLAY)
                        .on_hover_text("Use this color")
                        .on_hover_cursor(CursorIcon::PointingHand)
                        .clicked()
                {
                    if ctx.app.picker.hex_color.len() < 6 {
                        append_global_error("Enter a color first (ex. ab12ff #1200ff)".to_owned());
                    } else if let Some(color) =
                        Color::from_hex(ctx.app.picker.hex_color.trim_start_matches('#'))
                    {
                        ctx.app.picker.set_cur_color(color);
                    } else {
                        append_global_error("The entered hex color is not valid".to_owned());
                    }
                }
                if ui
                    .button(icon::ADD)
                    .on_hover_text(ADD_DESCR)
                    .on_hover_cursor(CursorIcon::Copy)
                    .clicked()
                {
                    ctx.app.add_cur_color()
                }
            });
        });
    }

    fn gradient_box(
        &mut self,
        ctx: &mut FrameCtx,
        gradient: &Gradient,
        size: Vec2,
        ui: &mut Ui,
        border: bool,
    ) {
        let tex_allocator = &mut ctx.tex_allocator();
        let _ = render_gradient(
            ui,
            tex_allocator,
            ctx.tex_manager,
            gradient,
            size,
            None,
            border,
        );
    }

    fn top_panel(&mut self, ctx: &mut FrameCtx<'_>) {
        let frame = egui::Frame {
            fill: if ctx.egui.style().visuals.dark_mode {
                *D_BG_00
            } else {
                *L_BG_0
            },
            inner_margin: Margin::symmetric(15., 10.),
            ..Default::default()
        };
        egui::TopBottomPanel::top("top panel")
            .frame(frame)
            .show(ctx.egui, |ui| {
                self.top_ui(ctx, ui);
            });
    }

    fn top_ui(&mut self, ctx: &mut FrameCtx<'_>, ui: &mut Ui) {
        ui.horizontal(|ui| {
            macro_rules! add_button_if {
                ($text:expr, $condition:expr, $block:tt) => {
                    add_button_if!($text, $condition, $block, $block);
                };
                ($text:expr, $condition:expr, $block_a:tt, $block_b:tt) => {
                    if $condition {
                        if ui
                            .button($text)
                            .on_hover_cursor(CursorIcon::PointingHand)
                            .clicked()
                        $block_a;
                    } else {
                        let btn = Button::new($text).fill(Rgba::from_black_alpha(0.));
                        if ui
                            .add(btn)
                            .on_hover_cursor(CursorIcon::PointingHand)
                            .clicked()
                        $block_b;
                    }
                };
            }
            add_button_if!(
                "picker",
                matches!(ctx.app.central_panel_tab, CentralPanelTab::Picker),
                {
                    ctx.app.central_panel_tab = CentralPanelTab::Picker;
                }
            );
            add_button_if!(
                "palettes",
                matches!(ctx.app.central_panel_tab, CentralPanelTab::Palettes),
                {
                    ctx.app.central_panel_tab = CentralPanelTab::Palettes;
                    ctx.app.sidepanel.show = false;
                }
            );

            ui.add_space(DOUBLE_SPACE);

            add_button_if!(
                "hues",
                self.windows.hues.is_open,
                { self.windows.hues.is_open = false },
                { self.windows.hues.is_open = true }
            );
            add_button_if!(
                "shades",
                self.windows.shades.is_open,
                { self.windows.shades.is_open = false },
                { self.windows.shades.is_open = true }
            );
            add_button_if!(
                "tints",
                self.windows.tints.is_open,
                { self.windows.tints.is_open = false },
                { self.windows.tints.is_open = true }
            );

            ui.with_layout(Layout::right_to_left(eframe::emath::Align::Center), |ui| {
                if ui
                    .button(icon::HELP)
                    .on_hover_text("Show help")
                    .on_hover_cursor(CursorIcon::Help)
                    .clicked()
                {
                    self.windows.help.toggle_window();
                }
                if ui
                    .button(icon::EXPAND)
                    .on_hover_text("Show/hide side panel")
                    .on_hover_cursor(CursorIcon::ResizeHorizontal)
                    .clicked()
                {
                    ctx.app.sidepanel.show = !ctx.app.sidepanel.show;
                }
                if ui
                    .button(icon::SETTINGS)
                    .on_hover_text("Settings")
                    .on_hover_cursor(CursorIcon::PointingHand)
                    .clicked()
                {
                    self.windows.settings.show = true;
                }
            });
        });
    }

    fn display_windows(&mut self, ctx: &mut FrameCtx<'_>) {
        self.windows.settings.display(ctx);
        self.windows.settings.custom_formats_window.display(
            &mut ctx.app.settings,
            ctx.egui,
            ctx.app.picker.current_color,
        );
        self.windows.settings.palette_formats_window.display(ctx);
        if let Err(e) = self.windows.export.display(ctx) {
            append_global_error(e);
        }

        self.shades_window(ctx);
        self.tints_window(ctx);
        self.hues_window(ctx);
        self.windows.help.display(ctx.egui);
    }

    fn central_panel(&mut self, ctx: &mut FrameCtx<'_>) {
        let _frame = egui::Frame {
            fill: if ctx.egui.style().visuals.dark_mode {
                *D_BG_0
            } else {
                *L_BG_2
            },

            inner_margin: Margin {
                left: 10.,
                top: 5.,
                right: 0.,
                bottom: 0.,
            },
            ..Default::default()
        };
        egui::CentralPanel::default()
            .frame(_frame)
            .show(ctx.egui, |ui| match ctx.app.central_panel_tab {
                CentralPanelTab::Picker => self.picker_ui(ctx, ui),
                CentralPanelTab::Palettes => self.palettes_ui(ctx, ui),
            });
    }

    fn picker_ui(&mut self, ctx: &mut FrameCtx<'_>, ui: &mut Ui) {
        let mut top_padding = 0.;
        let mut err_idx = 0;
        self.display_errors.retain(|e| {
            let elapsed = crate::elapsed(e.timestamp());
            if elapsed >= ERROR_DISPLAY_DURATION {
                false
            } else {
                if let Some(rsp) = egui::Window::new("Error")
                    .collapsible(false)
                    .id(Id::new(format!("err_ntf_{err_idx}")))
                    .anchor(
                        egui::Align2::RIGHT_TOP,
                        (-ctx.app.sidepanel.box_width - 25., top_padding),
                    )
                    .hscroll(true)
                    .fixed_size((ctx.app.sidepanel.box_width, 50.))
                    .show(ui.ctx(), |ui| {
                        let label =
                            Label::new(RichText::new(e.message()).color(Color32::RED)).wrap(true);
                        ui.add(label);
                    })
                {
                    top_padding += rsp.response.rect.height() + 6.;
                    err_idx += 1;
                };
                true
            }
        });

        ui.horizontal(|ui| {
            ui.add_space(HALF_SPACE);
            if ctx.app.settings.harmony_display_box {
                self.display_harmonies(ctx, ui);
            }
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    let cb = ColorBox::builder()
                        .size((CURRENT_COLOR_BOX_SIZE, CURRENT_COLOR_BOX_SIZE))
                        .color(ctx.app.picker.current_color)
                        .label(true)
                        .hover_help(COLORBOX_PICK_TOOLTIP)
                        .border(true)
                        .build();
                    cb.display(ctx, ui);
                    ui.label("Current");
                    if ui
                        .button(icon::COPY)
                        .on_hover_text("Copy color to clipboard")
                        .on_hover_cursor(CursorIcon::Alias)
                        .clicked()
                    {
                        if let Err(e) = save_to_clipboard(
                            ctx.app.clipboard_color(&ctx.app.picker.current_color),
                        ) {
                            append_global_error(format!(
                                "Failed to save color to clipboard - {}",
                                e
                            ));
                        }
                    }
                    if ui
                        .button(icon::ADD)
                        .on_hover_text(ADD_DESCR)
                        .on_hover_cursor(CursorIcon::Copy)
                        .clicked()
                    {
                        ctx.app.add_cur_color();
                    }
                });

                self.zoom_picker.display(ctx, ui);
            });
        });

        ui.add_space(SPACE);

        ScrollArea::vertical()
            .id_source("picker scroll")
            .show(ui, |ui| {
                ui.separator();
                self.harmonies_ctl_ui(ctx, ui);
                ui.separator();
                self.sliders(ctx, ui);
                ui.separator();
                self.hex_input(ctx, ui);
                let mut available_space = ui.available_size_before_wrap();
                if ctx.app.sidepanel.show {
                    available_space.x -= ctx.app.sidepanel.response_size.x;
                }
                ui.allocate_space(available_space);
            });
    }

    fn sliders(&mut self, ctx: &mut FrameCtx<'_>, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let sliders = [
                "RGB", "CMYK", "HSV", "HSL", "LUV", "LCH_UV", "LAB", "LCH_AB",
            ];
            for (i, name) in sliders.into_iter().enumerate() {
                if ui
                    .selectable_label(self.selected_slider == i as u8, name)
                    .clicked()
                {
                    self.selected_slider = i as u8;
                }
            }
        });
        match self.selected_slider {
            0 => ctx.app.picker.rgb_sliders(ui),
            1 => ctx.app.picker.cmyk_sliders(ui),
            2 => ctx.app.picker.hsv_sliders(ui),
            3 => ctx.app.picker.hsl_sliders(ui),
            4 => ctx.app.picker.luv_sliders(ui),
            5 => ctx.app.picker.lch_uv_sliders(ui),
            6 => ctx.app.picker.lab_sliders(ui),
            7 => ctx.app.picker.lch_ab_sliders(ui),
            _ => {}
        }
    }
}
