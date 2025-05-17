mod palette;
mod scheme;
pub mod window;

use crate::{
    color::{Color, ColorHarmony, Gradient},
    context::{AppCtx, FrameCtx},
    error::{DisplayError, ERROR_STACK, append_global_error},
    keybinding::{KeyBindings, default_keybindings},
    render::{TextureManager, render_gradient},
    save_to_clipboard,
    screen_size::ScreenSize,
    settings::{self},
    ui::{
        HALF_SPACE, SPACE,
        colorbox::{COLORBOX_PICK_TOOLTIP, ColorBox},
        colors::*,
        dark_visuals, icon,
    },
    zoom_picker::ZoomPicker,
};
use window::{ExportWindow, HelpWindow, HuesWindow, SettingsWindow, ShadesWindow, TintsWindow};

use eframe::{CreationContext, Storage};
use egui::{
    Button, Color32, CursorIcon, Id, Label, Layout, Margin, Rgba, RichText, ScrollArea, Ui, Vec2,
    Visuals,
};
use once_cell::sync::{Lazy, OnceCell};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

static ADD_DESCR: &str = "Add this color to saved colors";
static ERROR_DISPLAY_DURATION: u64 = 20;

pub static KEYBINDINGS: Lazy<KeyBindings> = Lazy::new(default_keybindings);
pub static DARK_VISUALS: Lazy<Visuals> = Lazy::new(dark_visuals);
pub static CONTEXT: OnceCell<RwLock<AppCtx>> = OnceCell::new();
pub static TEXTURE_MANAGER: Lazy<RwLock<TextureManager>> =
    Lazy::new(|| RwLock::new(TextureManager::default()));

pub const CURRENT_COLOR_BOX_SIZE: f32 = 40.0;

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum CentralPanelTab {
    Picker,
    Palettes,
    Hues,
    Shades,
    Tints,
    Settings,
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
            Arc::new(egui::FontData::from_static(include_bytes!(
                "../../assets/fonts/FiraCode/FiraCode-Regular.ttf"
            ))),
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
            inner_margin: Margin::symmetric(15, 10),
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
                }
            );
            add_button_if!(
                "hues",
                matches!(ctx.app.central_panel_tab, CentralPanelTab::Hues),
                {
                    ctx.app.central_panel_tab = CentralPanelTab::Hues;
                }
            );
            add_button_if!(
                "shades",
                matches!(ctx.app.central_panel_tab, CentralPanelTab::Shades),
                {
                    ctx.app.central_panel_tab = CentralPanelTab::Shades;
                }
            );
            add_button_if!(
                "tints",
                matches!(ctx.app.central_panel_tab, CentralPanelTab::Tints),
                {
                    ctx.app.central_panel_tab = CentralPanelTab::Tints;
                }
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
                let mut text = egui::RichText::new(icon::SETTINGS);
                if matches!(ctx.app.central_panel_tab, CentralPanelTab::Settings) {
                    text = text.color(egui::Color32::YELLOW);
                }
                if ui
                    .button(text)
                    .on_hover_text("Settings")
                    .on_hover_cursor(CursorIcon::PointingHand)
                    .clicked()
                {
                    ctx.app.central_panel_tab = CentralPanelTab::Settings;
                }
            });
        });
    }

    fn display_settings_stuff(&mut self, ctx: &mut FrameCtx<'_>, ui: &mut Ui) {
        self.windows.settings.display(ctx, ui);
        if let Err(e) = self.windows.export.display(ctx) {
            append_global_error(e);
        }
    }

    fn central_panel(&mut self, ctx: &mut FrameCtx<'_>) {
        let _frame = egui::Frame {
            fill: if ctx.egui.style().visuals.dark_mode {
                *D_BG_0
            } else {
                *L_BG_2
            },

            inner_margin: Margin {
                left: 10,
                top: 5,
                right: 0,
                bottom: 0,
            },
            ..Default::default()
        };
        egui::CentralPanel::default()
            .frame(_frame)
            .show(ctx.egui, |ui| match ctx.app.central_panel_tab {
                CentralPanelTab::Picker => self.picker_ui(ctx, ui),
                CentralPanelTab::Palettes => self.palettes_ui(ctx, ui),
                CentralPanelTab::Hues => self.hues_window(ctx, ui),
                CentralPanelTab::Shades => self.shades_window(ctx, ui),
                CentralPanelTab::Tints => self.tints_window(ctx, ui),
                CentralPanelTab::Settings => self.display_settings_stuff(ctx, ui),
            });
        self.windows.help.display(ctx.egui);
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
                            Label::new(RichText::new(e.message()).color(Color32::RED)).wrap();
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
                    let hover_ui = |ui: &mut egui::Ui| {
                        ui.label(
                            egui::RichText::new("Copy to clipboard (ctrl+c)")
                                .color(egui::Color32::GRAY),
                        );
                        ui.label(ctx.app.clipboard_color(&ctx.app.picker.current_color));
                    };
                    if (ui
                        .button(icon::COPY)
                        .on_hover_ui(hover_ui)
                        .on_hover_cursor(CursorIcon::Alias)
                        .clicked()
                        || ui.input(|inp| {
                            inp.events.iter().any(|ev| matches!(ev, egui::Event::Copy))
                        }))
                        && let Err(e) = save_to_clipboard(
                            ctx.app.clipboard_color(&ctx.app.picker.current_color),
                        )
                    {
                        append_global_error(format!("Failed to save color to clipboard - {e}"));
                    }
                    if ui
                        .button(icon::ADD)
                        .on_hover_text(ADD_DESCR)
                        .on_hover_cursor(CursorIcon::Copy)
                        .clicked()
                    {
                        ctx.app.add_cur_color();
                    }
                    let re = ui.button(icon::EDIT).on_hover_text("Enter hex color");
                    let color_edit_id = egui::Id::new("color-edit-popup");
                    let mut just_clicked = false;
                    if re.clicked() {
                        ui.memory_mut(|mem| mem.open_popup(color_edit_id));
                        just_clicked = true;
                    }
                    custom_popup_below_widget(ui, color_edit_id, &re, 100.0, |ui| {
                        ui.horizontal(|ui| {
                            let resp = ui.text_edit_singleline(&mut ctx.app.picker.hex_color);
                            if just_clicked {
                                resp.request_focus();
                            }
                            if (resp.lost_focus()
                                && ui.input(|inp| inp.key_pressed(egui::Key::Enter)))
                                || ui
                                    .button(icon::PLAY)
                                    .on_hover_text("Use this color")
                                    .on_hover_cursor(CursorIcon::PointingHand)
                                    .clicked()
                            {
                                if ctx.app.picker.hex_color.len() < 6 {
                                    append_global_error(
                                        "Enter a color first (ex. ab12ff #1200ff)".to_owned(),
                                    );
                                } else if let Some(color) = Color::from_hex(
                                    ctx.app.picker.hex_color.trim_start_matches('#'),
                                ) {
                                    ctx.app.picker.set_cur_color(color);
                                } else {
                                    append_global_error(
                                        "The entered hex color is not valid".to_owned(),
                                    );
                                }
                                ui.memory_mut(|mem| mem.close_popup());
                            }
                            if ui
                                .button(icon::ADD)
                                .on_hover_text(ADD_DESCR)
                                .on_hover_cursor(CursorIcon::Copy)
                                .clicked()
                            {
                                ctx.app.add_cur_color()
                            }
                        })
                    });
                });

                self.zoom_picker.display(ctx, ui);
            });
        });

        ui.add_space(SPACE);

        ScrollArea::vertical()
            .id_salt("picker scroll")
            .show(ui, |ui| {
                ui.separator();
                self.harmonies_ctl_ui(ctx, ui);
                ui.separator();
                self.sliders(ctx, ui);
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

pub fn custom_popup_below_widget<R>(
    ui: &Ui,
    popup_id: Id,
    widget_response: &egui::Response,
    width: f32,
    add_contents: impl FnOnce(&mut Ui) -> R,
) -> Option<R> {
    if ui.memory(|mem| mem.is_popup_open(popup_id)) {
        let (pos, pivot) = (widget_response.rect.left_bottom(), egui::Align2::LEFT_TOP);
        let re = egui::Area::new(popup_id)
            .order(egui::Order::Foreground)
            .constrain(true)
            .fixed_pos(pos)
            .pivot(pivot)
            .show(ui.ctx(), |ui| {
                let frame = egui::Frame::popup(ui.style());
                frame
                    .show(ui, |ui| {
                        ui.with_layout(Layout::top_down_justified(egui::Align::LEFT), |ui| {
                            ui.set_width(width);
                            add_contents(ui)
                        })
                        .inner
                    })
                    .inner
            });
        if widget_response.clicked_elsewhere() && re.response.clicked_elsewhere() {
            ui.memory_mut(|mem| mem.close_popup());
        }
        Some(re.inner)
    } else {
        None
    }
}
