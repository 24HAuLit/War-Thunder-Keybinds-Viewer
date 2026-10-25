use std::collections::BTreeSet;

use eframe::egui::{self, RichText};
use wt_keybind_viewer::bindings::{
    analyzer::Analysis,
    keymap::{Key, Layout, key_label, keyboard_rows},
    model::Profile,
};

use super::{ACCENT, MULTIPLE, MUTED, SELECTED, UNLISTED, USED, binding_card};

/// Returns an action ID when a details link requests a switch to the action list.
pub fn show(
    ui: &mut egui::Ui,
    profile: &Profile,
    analysis: &Analysis,
    layout: Layout,
    selected: &mut Option<i32>,
) -> Option<String> {
    let mut action = None;
    egui::ScrollArea::vertical()
        .id_salt("keyboard_page")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                legend(ui, USED, "1 explicit entry");
                legend(ui, MULTIPLE, "Multiple entries (not necessarily a conflict)");
                legend(ui, UNLISTED, "No explicit entry");
                legend(ui, SELECTED, "Selected");
            });
            ui.small("Select a key to inspect every containing combination. Scroll horizontally on narrow windows; key sizes remain readable.");
            ui.add_space(6.0);
            let rows = keyboard_rows();
            // Constrain the viewport, not the keyboard: otherwise a wide child can
            // expand the parent and push both details and scrollbars off-screen.
            egui::ScrollArea::horizontal()
                .id_salt("keyboard_horizontal")
                .max_width(ui.available_width())
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.horizontal_top(|ui| {
                        ui.vertical(|ui| {
                            ui.small("MAIN / FUNCTION KEYS");
                            for row in &rows[..6] {
                                key_row(ui, row, analysis, layout, selected);
                            }
                        });
                        ui.add_space(10.0);
                        ui.vertical(|ui| {
                            ui.small("NAVIGATION");
                            for row in &rows[6..10] {
                                key_row(ui, row, analysis, layout, selected);
                            }
                        });
                        ui.add_space(10.0);
                        ui.vertical(|ui| {
                            ui.small("NUMPAD");
                            for row in &rows[10..] {
                                key_row(ui, row, analysis, layout, selected);
                            }
                        });
                    });
                });
            let drawn: BTreeSet<_> = rows.iter().flatten().map(|key| key.code).collect();
            let extra: Vec<_> = analysis.by_key.keys().filter(|code| !drawn.contains(*code)).copied().collect();
            if !extra.is_empty() {
                ui.add_space(8.0);
                ui.strong("Other keyboard codes in this export");
                ui.small("Regional, media, nonvisual, and unknown codes are retained without remapping.");
                ui.horizontal_wrapped(|ui| {
                    for code in extra {
                        let count = analysis.indices_for_key(code).len();
                        let text = format!("{} · code {code} · {count}", key_label(code, layout));
                        if ui.add(egui::Button::new(RichText::new(text).color(egui::Color32::WHITE))
                            .fill(key_color(count, *selected == Some(code))))
                            .clicked()
                        {
                            *selected = Some(code);
                        }
                    }
                });
            }
            ui.add_space(10.0);
            ui.separator();
            if let Some(code) = *selected {
                ui.horizontal_wrapped(|ui| {
                    ui.heading(key_label(code, layout));
                    ui.label(RichText::new(format!("Physical DIK code {code} ({code:#x})")).color(MUTED));
                    if ui.small_button("Clear selection").clicked() {
                        *selected = None;
                    }
                });
                let indices = analysis.indices_for_key(code);
                ui.label(format!("{} explicit entries contain this key", indices.len()));
                if indices.is_empty() {
                    ui.label("No explicit entry in this export. This key may still be assigned by an unresolved preset or game default.");
                }
                for &index in indices {
                    if binding_card(ui, profile, analysis, index, layout, true) {
                        action = Some(profile.bindings[index].action.clone());
                    }
                    ui.add_space(3.0);
                }
            } else {
                ui.label(RichText::new("Select a key to explore its actions and full combinations.").color(ACCENT));
            }
        });
    action
}

fn key_row(
    ui: &mut egui::Ui,
    row: &[Key],
    analysis: &Analysis,
    layout: Layout,
    selected: &mut Option<i32>,
) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        for key in row {
            let count = analysis.indices_for_key(key.code).len();
            let label = key_label(key.code, layout);
            let short = short_label(&label);
            let text = if count == 0 { short } else { format!("{short}\n{count}") };
            let width = 54.0 * key.width + 4.0 * (key.width - 1.0);
            let response = ui.add_sized(
                [width, 56.0],
                egui::Button::new(RichText::new(text).size(12.0).color(egui::Color32::WHITE))
                    .fill(key_color(count, *selected == Some(key.code)))
                    .stroke(if *selected == Some(key.code) {
                        egui::Stroke::new(2.0_f32, ACCENT)
                    } else {
                        egui::Stroke::new(1.0_f32, egui::Color32::from_white_alpha(25))
                    }),
            );
            if response.clicked() {
                *selected = Some(key.code);
            }
            response.on_hover_text(format!(
                "{label}\nPhysical DIK code {} ({:#x})\n{count} explicit entries\nCounts include full combinations, not just this key alone.\nNo explicit entry does not mean free in game.",
                key.code, key.code,
            ));
        }
    });
}

fn key_color(count: usize, selected: bool) -> egui::Color32 {
    if selected {
        SELECTED
    } else {
        match count {
            0 => UNLISTED,
            1 => USED,
            _ => MULTIPLE,
        }
    }
}

fn short_label(label: &str) -> String {
    match label {
        "Escape" => "Esc",
        "Backspace" => "Backspace",
        "Caps Lock" => "Caps Lock",
        "Print Screen" => "PrtSc",
        "Scroll Lock" => "Scroll",
        "Num Lock" => "Num",
        "Page Up" => "PgUp",
        "Page Down" => "PgDn",
        "Insert" => "Ins",
        "Delete" => "Del",
        "Numpad Enter" => "Num Ent",
        _ => {
            return label
                .replace("Numpad ", "Num ")
                .replace("Left ", "L ")
                .replace("Right ", "R ");
        }
    }
    .into()
}

fn legend(ui: &mut egui::Ui, color: egui::Color32, label: &str) {
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(11.0, 11.0), egui::Sense::hover());
        ui.painter().rect_filled(rect, 3.0, color);
        ui.small(label);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_takes_precedence_over_usage() {
        for count in [0, 1, 20] {
            assert_eq!(key_color(count, true), SELECTED);
        }
        assert_eq!(key_color(0, false), UNLISTED);
        assert_eq!(key_color(1, false), USED);
        assert_eq!(key_color(2, false), MULTIPLE);
    }
}
