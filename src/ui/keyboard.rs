use std::collections::BTreeSet;

use eframe::egui::{self, RichText};
use wt_keybind_viewer::bindings::{
    analyzer::Analysis,
    keymap::{Key, KeyShape, Layout, key_label, keyboard_keys},
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
                legend(ui, USED, "1 entry (export or preset)");
                legend(ui, MULTIPLE, "Multiple entries (not necessarily a conflict)");
                legend(ui, UNLISTED, "No displayed entry");
                legend(ui, SELECTED, "Selected");
            });
            ui.small("Select a key to inspect every containing combination. Scroll horizontally on narrow windows; key sizes remain readable.");
            ui.add_space(6.0);
            let keys = keyboard_keys();
            // Constrain the viewport, not the keyboard: otherwise a wide child can
            // expand the parent and push both details and scrollbars off-screen.
            egui::ScrollArea::horizontal()
                .id_salt("keyboard_horizontal")
                .max_width(ui.available_width())
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    keyboard(ui, &keys, analysis, layout, selected);
                });
            let drawn: BTreeSet<_> = keys.iter().map(|key| key.code).collect();
            let extra: Vec<_> = analysis.by_key.keys().filter(|code| !drawn.contains(*code)).copied().collect();
            if !extra.is_empty() {
                ui.add_space(8.0);
                ui.strong("Other keyboard codes in this profile");
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
                ui.label(format!("{} entries contain this key (export + resolved presets)", indices.len()));
                if indices.is_empty() {
                    ui.label("No displayed entry. This key may still be assigned by an unresolved preset or game default.");
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

const KEY_PITCH: egui::Vec2 = egui::vec2(58.0, 60.0);
const KEY_GAP: f32 = 4.0;
const HEADER_HEIGHT: f32 = 22.0;

fn key_rect(origin: egui::Pos2, key: &Key) -> egui::Rect {
    egui::Rect::from_min_size(
        origin + egui::vec2(key.x * KEY_PITCH.x, key.y * KEY_PITCH.y),
        egui::vec2(
            key.width * KEY_PITCH.x - KEY_GAP,
            key.height * KEY_PITCH.y - KEY_GAP,
        ),
    )
}

// ISO Enter is a single key with a quarter-unit notch for the neighbouring key.
fn iso_enter_parts(rect: egui::Rect) -> (egui::Rect, egui::Rect) {
    let top = egui::Rect::from_min_max(
        rect.min,
        egui::pos2(rect.right(), rect.top() + KEY_PITCH.y - KEY_GAP),
    );
    let stem = egui::Rect::from_min_max(
        egui::pos2(rect.left() + 0.25 * KEY_PITCH.x, rect.top()),
        rect.max,
    );
    (top, stem)
}

fn key_contains(key: &Key, rect: egui::Rect, pos: egui::Pos2) -> bool {
    match key.shape {
        KeyShape::Rectangle => rect.contains(pos),
        KeyShape::IsoEnter => {
            let (top, stem) = iso_enter_parts(rect);
            top.contains(pos) || stem.contains(pos)
        }
    }
}

fn keyboard(
    ui: &mut egui::Ui,
    keys: &[Key],
    analysis: &Analysis,
    layout: Layout,
    selected: &mut Option<i32>,
) {
    let width = keys.iter().map(|key| key.x + key.width).fold(0.0, f32::max);
    let height = keys
        .iter()
        .map(|key| key.y + key.height)
        .fold(0.0, f32::max);
    let (bounds, _) = ui.allocate_exact_size(
        egui::vec2(
            width * KEY_PITCH.x - KEY_GAP,
            height * KEY_PITCH.y - KEY_GAP + HEADER_HEIGHT,
        ),
        egui::Sense::hover(),
    );
    for (x, label) in [
        (0.0, "MAIN / FUNCTION KEYS"),
        (15.5, "NAVIGATION"),
        (19.0, "NUMPAD"),
    ] {
        ui.painter().text(
            bounds.min + egui::vec2(x * KEY_PITCH.x, 0.0),
            egui::Align2::LEFT_TOP,
            label,
            egui::TextStyle::Small.resolve(ui.style()),
            MUTED,
        );
    }
    let origin = bounds.min + egui::vec2(0.0, HEADER_HEIGHT);
    for key in keys {
        let rect = key_rect(origin, key);
        let count = analysis.indices_for_key(key.code).len();
        let label = key_label(key.code, layout);
        let short = short_label(&label);
        let text = if count == 0 {
            short
        } else {
            format!("{short}\n{count}")
        };
        let fill = key_color(count, *selected == Some(key.code));
        let stroke = if *selected == Some(key.code) {
            egui::Stroke::new(2.0_f32, ACCENT)
        } else {
            egui::Stroke::new(1.0_f32, egui::Color32::from_white_alpha(25))
        };
        let response = match key.shape {
            KeyShape::Rectangle => ui.put(
                rect,
                egui::Button::new(RichText::new(text).size(12.0).color(egui::Color32::WHITE))
                    .fill(fill)
                    .stroke(stroke),
            ),
            KeyShape::IsoEnter => {
                let response = ui.interact(rect, ui.id().with(key.code), egui::Sense::click());
                let (top, stem) = iso_enter_parts(rect);
                // Two filled rectangles avoid triangulating a concave polygon.
                ui.painter().rect_filled(top, 0.0, fill);
                ui.painter().rect_filled(stem, 0.0, fill);
                let stroke = if response.has_focus() || response.hovered() {
                    egui::Stroke::new(stroke.width, ACCENT)
                } else {
                    stroke
                };
                ui.painter().add(egui::Shape::closed_line(
                    vec![
                        top.left_top(),
                        top.right_top(),
                        stem.right_bottom(),
                        stem.left_bottom(),
                        egui::pos2(stem.left(), top.bottom()),
                        top.left_bottom(),
                    ],
                    stroke,
                ));
                ui.painter().text(
                    stem.center(),
                    egui::Align2::CENTER_CENTER,
                    text,
                    egui::FontId::proportional(12.0),
                    egui::Color32::WHITE,
                );
                response.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), &label)
                });
                response
            }
        };
        let pointer_on_key = ui
            .input(|input| input.pointer.hover_pos())
            .is_some_and(|pos| key_contains(key, rect, pos));
        if response.clicked() && (pointer_on_key || response.has_focus()) {
            *selected = Some(key.code);
        }
        if pointer_on_key {
            response.on_hover_text(format!(
                "{label}\nPhysical DIK code {} ({:#x})\n{count} entries (export + resolved presets)\nCounts include full combinations, not just this key alone.\nNo displayed entry does not mean free in game.",
                key.code, key.code,
            ));
        }
    }
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
    fn pixel_geometry_preserves_alignment_and_tall_key_bounds() {
        let keys = keyboard_keys();
        let rect = |code| {
            key_rect(
                egui::Pos2::ZERO,
                keys.iter().find(|key| key.code == code).unwrap(),
            )
        };
        for (upper, lower) in [(183, 210), (70, 199), (197, 201)] {
            assert_eq!(rect(upper).left(), rect(lower).left());
            assert!(rect(upper).bottom() < rect(lower).top());
        }
        assert_eq!(rect(200).center().x, rect(208).center().x);
        assert_eq!(
            rect(200).center().x,
            (rect(203).center().x + rect(205).center().x) / 2.0
        );
        for (tall, top, bottom) in [(78, 73, 77), (156, 81, 83)] {
            assert_eq!(rect(tall).top(), rect(top).top());
            assert_eq!(rect(tall).bottom(), rect(bottom).bottom());
            assert_eq!(rect(tall).height(), 2.0 * KEY_PITCH.y - KEY_GAP);
        }
        let enter = keys.iter().find(|key| key.code == 28).unwrap();
        let (top, stem) = iso_enter_parts(rect(28));
        assert!(key_contains(
            enter,
            rect(28),
            top.left_center() + egui::vec2(1.0, 0.0)
        ));
        assert!(key_contains(
            enter,
            rect(28),
            stem.center_bottom() - egui::vec2(0.0, 1.0)
        ));
        let notch = egui::pos2(top.left() + 1.0, rect(43).center().y);
        assert!(rect(43).contains(notch));
        assert!(!key_contains(enter, rect(28), notch));
    }

    fn keyboard_frame(
        ctx: &egui::Context,
        layout: Layout,
        selected: &mut Option<i32>,
        events: Vec<egui::Event>,
    ) -> egui::Pos2 {
        let profile = wt_keybind_viewer::bindings::analyzer::import("controls {}").unwrap();
        let analysis = wt_keybind_viewer::bindings::analyzer::analyze(&profile);
        let mut origin = egui::Pos2::ZERO;
        let _ = ctx.run(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1500.0, 600.0),
                )),
                events,
                ..Default::default()
            },
            |ctx| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    origin = ui.cursor().min + egui::vec2(0.0, HEADER_HEIGHT);
                    keyboard(ui, &keyboard_keys(), &analysis, layout, selected);
                });
            },
        );
        origin
    }

    #[test]
    fn tall_keys_and_iso_neighbour_are_clickable_in_both_layouts() {
        let keys = keyboard_keys();
        for layout in [Layout::Qwerty, Layout::Azerty] {
            let ctx = egui::Context::default();
            let mut selected = None;
            let origin = keyboard_frame(&ctx, layout, &mut selected, vec![]);
            let rect = |code| key_rect(origin, keys.iter().find(|key| key.code == code).unwrap());
            for code in [78, 156, 28, 43, 200, 183, 70, 197, 86] {
                let mut positions = vec![rect(code).center()];
                if matches!(code, 78 | 156 | 28) {
                    positions = vec![
                        rect(code).center_top() + egui::vec2(0.0, 10.0),
                        rect(code).center_bottom() - egui::vec2(0.0, 10.0),
                    ];
                } else if code == 43 {
                    // This point is inside Enter's bounding box, but in its notch.
                    positions.push(egui::pos2(rect(28).left() + 1.0, rect(43).center().y));
                }
                for pos in positions {
                    selected = None;
                    keyboard_frame(
                        &ctx,
                        layout,
                        &mut selected,
                        vec![egui::Event::PointerMoved(pos)],
                    );
                    for pressed in [true, false] {
                        keyboard_frame(
                            &ctx,
                            layout,
                            &mut selected,
                            vec![egui::Event::PointerButton {
                                pos,
                                button: egui::PointerButton::Primary,
                                pressed,
                                modifiers: egui::Modifiers::NONE,
                            }],
                        );
                    }
                    assert_eq!(selected, Some(code), "{layout:?}: click at {pos:?}");
                }
            }
        }
    }

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
