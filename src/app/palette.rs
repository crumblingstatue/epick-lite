use crate::{
    app::App,
    color::NamedPalette,
    context::FrameCtx,
    save_to_clipboard,
    ui::{
        colorbox::{ColorBox, COLORBOX_DRAG_TOOLTIP},
        drop_target, icon, SPACE,
    },
};

use egui::{CursorIcon, Id, Label, RichText, ScrollArea, Ui};

enum UiAction {
    DeleteColor { pal_idx: usize, col_idx: usize },
    Swap { a: ColorIdx, b: ColorIdx },
    RemPush { rem_idx: ColorIdx, push_idx: usize },
}

#[derive(Clone, Copy)]
struct ColorIdx {
    pal_idx: usize,
    col_idx: usize,
}

impl App {
    pub fn palettes_ui(&mut self, ctx: &mut FrameCtx<'_>, ui: &mut Ui) {
        ScrollArea::new([true, true]).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.checkbox(
                    &mut ctx.app.palettes_tab_display_label,
                    "Display color labels",
                );
            });
            ui.add(
                egui::Slider::new(&mut ctx.app.palettes_tab_color_size, 25.0..=100.)
                    .clamping(egui::SliderClamping::Always)
                    .text("color size"),
            );
            ui.horizontal(|ui| {
                if ui
                    .button(icon::ADD)
                    .on_hover_text("Add a new palette")
                    .clicked()
                {
                    ctx.app.palettes.append_empty();
                }
            });
            ui.add_space(SPACE);

            let current = ctx.app.palettes.current_idx();
            let mut ui_action = None;
            for (i, palette) in ctx.app.palettes.clone().iter().enumerate() {
                let active = current == i;
                self.display_palette(palette, i, active, ctx, ui, &mut ui_action);
            }
            if let Some(action) = ui_action {
                match action {
                    UiAction::DeleteColor { pal_idx, col_idx } => {
                        ctx.app.palettes.palettes[pal_idx]
                            .palette
                            .remove_pos(col_idx);
                    }
                    UiAction::Swap { a, b } => {
                        if a.pal_idx == b.pal_idx {
                            ctx.app.palettes.palettes[a.pal_idx]
                                .palette
                                .0
                                .swap(a.col_idx, b.col_idx);
                        } else {
                            let first = ctx.app.palettes.palettes[a.pal_idx]
                                .palette
                                .remove_pos(a.col_idx)
                                .unwrap();
                            let second = ctx.app.palettes.palettes[b.pal_idx]
                                .palette
                                .remove_pos(b.col_idx)
                                .unwrap();
                            ctx.app.palettes.palettes[a.pal_idx]
                                .palette
                                .insert(a.col_idx, second);
                            ctx.app.palettes.palettes[b.pal_idx]
                                .palette
                                .insert(b.col_idx, first);
                        }
                    }
                    UiAction::RemPush { rem_idx, push_idx } => {
                        let color = ctx.app.palettes.palettes[rem_idx.pal_idx]
                            .palette
                            .remove_pos(rem_idx.col_idx)
                            .unwrap();
                        ctx.app.palettes.palettes[push_idx].palette.0.push(color);
                    }
                }
            }
        });
    }

    fn display_palette(
        &mut self,
        palette: &NamedPalette,
        index: usize,
        active: bool,
        ctx: &mut FrameCtx<'_>,
        ui: &mut Ui,
        action: &mut Option<UiAction>,
    ) {
        ui.horizontal(|ui| {
            self.display_palette_buttons(palette, ctx, ui);
            let mut label = RichText::new(&palette.name);
            if active {
                label = label.strong().heading();
            }
            ui.vertical(|ui| {
                let re = ui.add(Label::new(label));
                if let Some(payload) = re.dnd_hover_payload::<ColorIdx>() {
                    ui.painter().rect_stroke(
                        re.rect,
                        2.0,
                        egui::Stroke::new(2.0, egui::Color32::WHITE),
                    );
                    if ui.input(|inp| inp.pointer.primary_released()) {
                        *action = Some(UiAction::RemPush {
                            rem_idx: *payload,
                            push_idx: index,
                        });
                    }
                } else if egui::DragAndDrop::has_payload_of_type::<ColorIdx>(ctx.egui) {
                    ui.painter().rect_stroke(
                        re.rect,
                        2.0,
                        egui::Stroke::new(2.0, egui::Color32::GRAY),
                    );
                }
                self.display_palette_colors(palette, index, ctx, ui, action);
                ui.add_space(SPACE);
            });
        });
    }

    fn display_palette_buttons(
        &mut self,
        palette: &NamedPalette,
        ctx: &mut FrameCtx<'_>,
        ui: &mut Ui,
    ) -> egui::InnerResponse<()> {
        ui.vertical(|ui| {
            if ui
                .button(icon::PLAY)
                .on_hover_text("Use this palette")
                .on_hover_cursor(CursorIcon::PointingHand)
                .clicked()
            {
                ctx.app.palettes.move_to_name(&palette.name);
            }
            if ui
                .button(icon::EXPORT)
                .on_hover_text("Export")
                .on_hover_cursor(CursorIcon::PointingHand)
                .clicked()
            {
                self.windows.export.show = true;
                self.windows.export.export_palette = Some(palette.clone());
            }
            if ui
                .button(icon::COPY)
                .on_hover_text("Copy all colors to clipboard")
                .on_hover_cursor(CursorIcon::Alias)
                .clicked()
            {
                let _ = save_to_clipboard(palette.display(
                    &ctx.app.settings.palette_clipboard_format,
                    ctx.app.settings.rgb_working_space,
                    ctx.app.settings.illuminant,
                ));
            }
            if ui
                .button(icon::DELETE)
                .on_hover_text("Delete this palette")
                .clicked()
            {
                ctx.app.palettes.remove(palette);
            }
        })
    }

    fn display_palette_colors(
        &mut self,
        palette: &NamedPalette,
        index: usize,
        ctx: &mut FrameCtx<'_>,
        ui: &mut Ui,
        action: &mut Option<UiAction>,
    ) -> egui::InnerResponse<()> {
        egui::Grid::new(&palette.name)
            .spacing((2.5, 0.))
            .show(ui, |ui| {
                let mut color_src_row = None;
                let mut color_dst_row = None;
                for (i, color) in palette.palette.iter().enumerate() {
                    let resp = drop_target(ui, true, |ui| {
                        let color_id = Id::new(&palette.name).with(i);
                        let cb = ColorBox::builder()
                            .size((
                                ctx.app.palettes_tab_color_size,
                                ctx.app.palettes_tab_color_size,
                            ))
                            .color(*color)
                            .label(ctx.app.palettes_tab_display_label)
                            .hover_help(COLORBOX_DRAG_TOOLTIP)
                            .build();
                        ui.vertical(|ui| {
                            if let Some(re) = cb.display(ctx, ui) {
                                let re = re.interact(egui::Sense::drag());
                                if re.drag_started_by(egui::PointerButton::Primary) {
                                    egui::DragAndDrop::set_payload(
                                        ctx.egui,
                                        ColorIdx {
                                            pal_idx: index,
                                            col_idx: i,
                                        },
                                    );
                                }
                                if ui.ctx().dragged_id() == Some(re.id)
                                    && egui::DragAndDrop::has_any_payload(ui.ctx())
                                {
                                    ui.painter().rect_stroke(
                                        re.rect,
                                        2.0,
                                        egui::Stroke::new(3.0, egui::Color32::WHITE),
                                    );
                                }
                                if let Some(idx) = re.dnd_hover_payload::<ColorIdx>() {
                                    ui.painter().rect_stroke(
                                        re.rect,
                                        2.0,
                                        egui::Stroke::new(3.0, egui::Color32::WHITE),
                                    );
                                    if ui.input(|inp| inp.pointer.primary_released()) {
                                        *action = Some(UiAction::Swap {
                                            a: *idx,
                                            b: ColorIdx {
                                                pal_idx: index,
                                                col_idx: i,
                                            },
                                        });
                                    }
                                }
                                re.context_menu(|ui| {
                                    if ui.button("Delete").clicked() {
                                        *action = Some(UiAction::DeleteColor {
                                            pal_idx: index,
                                            col_idx: i,
                                        });
                                        ui.close_menu();
                                    }
                                });
                            }
                        });
                        if ctx.egui.is_being_dragged(color_id) {
                            color_src_row = Some(i);
                        }
                    });
                    let is_being_dragged = ctx.egui.dragged_id().is_some();
                    if is_being_dragged && resp.response.hovered() {
                        color_dst_row = Some(i);
                    }
                }
                if let Some(src_row) = color_src_row {
                    if let Some(dst_row) = color_dst_row {
                        if ui.input(|inp| inp.pointer.any_released()) {
                            ctx.app.palettes.move_to_name(&palette.name);
                            let palette = &mut ctx.app.palettes.current_mut().palette;
                            if let Some(it) = palette.remove_pos(src_row) {
                                palette.insert(dst_row, it);
                            }
                        }
                    }
                }
            })
    }
}
