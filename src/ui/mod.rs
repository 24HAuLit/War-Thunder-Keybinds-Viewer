pub mod actions;
pub mod keyboard;

use eframe::egui::{self, Color32, RichText};
use wt_keybind_viewer::{
    bindings::{
        analyzer::Analysis,
        keymap::Layout,
        model::{IssueKind, Profile},
    },
    blk::{Block, Entry},
};

pub const ACCENT: Color32 = Color32::from_rgb(110, 177, 255);
pub const MUTED: Color32 = Color32::from_rgb(164, 177, 195);
pub const WARNING: Color32 = Color32::from_rgb(245, 184, 101);
pub const ERROR: Color32 = Color32::from_rgb(255, 137, 137);
pub const USED: Color32 = Color32::from_rgb(40, 103, 79);
pub const MULTIPLE: Color32 = Color32::from_rgb(139, 86, 36);
pub const UNLISTED: Color32 = Color32::from_rgb(54, 63, 78);
pub const SELECTED: Color32 = Color32::from_rgb(43, 99, 172);

pub fn configure(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = Color32::from_rgb(21, 27, 36);
    visuals.window_fill = Color32::from_rgb(27, 34, 45);
    visuals.faint_bg_color = Color32::from_rgb(30, 38, 50);
    visuals.selection.bg_fill = SELECTED;
    visuals.selection.stroke.color = Color32::WHITE;
    ctx.set_visuals(visuals);
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(9.0, 7.0);
    style.spacing.button_padding = egui::vec2(12.0, 7.0);
    ctx.set_style(style);
}

pub fn profile_metadata(ui: &mut egui::Ui, profile: &Profile) {
    egui::CollapsingHeader::new(format!(
        "Import notes & source metadata · {} warnings · {} base presets · {} includes",
        profile.warnings.len(), profile.preset_paths.len(), profile.includes.len(),
    ))
    .id_salt("profile_metadata")
    .show(ui, |ui| {
        egui::ScrollArea::vertical()
            .id_salt("metadata_scroll")
            .max_height(210.0)
            .show(ui, |ui| {
                for warning in &profile.warnings {
                    ui.colored_label(WARNING, warning);
                }
                ui.strong("Base preset paths declared in the export");
                if profile.preset_paths.is_empty() {
                    ui.small("None declared in this export. Game defaults are still not inferred.");
                }
                for path in &profile.preset_paths {
                    ui.monospace(path);
                }
                ui.strong("Applied embedded presets (base first)");
                                if profile.resolved_presets.is_empty() {
                                    ui.small("None applied. See warnings for unsupported references.");
                                }
                                for path in &profile.resolved_presets {
                                    ui.monospace(path);
                                }
                                ui.strong("Unresolved includes");
                if profile.includes.is_empty() {
                    ui.small("None declared.");
                }
                for path in &profile.includes {
                    ui.monospace(path);
                }
                ui.small("Only exact supported preset paths use embedded data. No referenced path is opened on disk or fetched from the network.");
                ui.strong("Raw document / device metadata");
                ui.small("Expand source blocks to inspect device names, IDs, settings, and all other fields. Original entry order and repeated properties are retained; whitespace and comments are not.");
                document_block(ui, &profile.document);
            });
    });
}

fn document_block(ui: &mut egui::Ui, block: &Block) {
    for (index, entry) in block.entries.iter().enumerate() {
        ui.push_id(index, |ui| match entry {
            Entry::Property(property) => {
                ui.monospace(property_text(
                    &property.name,
                    &property.kind,
                    &property.value,
                ));
            }
            Entry::Block(child) => {
                egui::CollapsingHeader::new(format!(
                    "{} · {} entries",
                    child.name,
                    child.entries.len()
                ))
                .show(ui, |ui| document_block(ui, child));
            }
        });
    }
}

fn property_text(name: &str, kind: &str, value: &str) -> String {
    let kind = if kind.is_empty() {
        String::new()
    } else {
        format!(":{kind}")
    };
    format!("{name}{kind} = {value:?}")
}

/// Shared details retain the exact source entry index, even for repeated action IDs.
pub fn binding_card(
    ui: &mut egui::Ui,
    profile: &Profile,
    analysis: &Analysis,
    index: usize,
    layout: Layout,
    action_link: bool,
) -> bool {
    let binding = &profile.bindings[index];
    let mut open_action = false;
    ui.push_id(index, |ui| {
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_wrapped(|ui| {
                ui.label(RichText::new(binding.display_name()).strong().color(ACCENT))
                    .on_hover_text(if binding.english_name().is_some() {
                        "Official English action name from the game."
                    } else {
                        "Unknown action ID: human-readable fallback, not an official game translation."
                    });
                ui.small(format!("entry #{} · {}", index + 1, binding.section.name()));
                if action_link && ui.small_button("Show action in list").clicked() {
                    open_action = true;
                }
            });
            ui.label(RichText::new(&binding.action).monospace().small().color(MUTED));
            ui.label(RichText::new(binding.combination(layout)).strong());
                        ui.small(binding_origin(binding));
            ui.horizontal_wrapped(|ui| {
                ui.label(format!("Context: {}", binding.context.as_deref().unwrap_or("Unknown (not declared)")));
                ui.label(format!("Devices: {}", devices(binding)));
            });
            if let Some(issues) = analysis.issues.get(index) {
                // Group by kind rather than emitting hundreds of repeated warning lines.
                for kind in [IssueKind::Duplicate, IssueKind::PotentialConflict, IssueKind::ContextUnknown] {
                    let matching: Vec<_> = issues.iter().filter(|issue| issue.kind == kind).collect();
                    if matching.is_empty() {
                        continue;
                    }
                    egui::CollapsingHeader::new(
                        RichText::new(format!("{} · {} related entries", kind.label(), matching.len()))
                            .color(WARNING),
                    )
                    .id_salt(format!("issue_{kind:?}"))
                    .show(ui, |ui| {
                        for issue in matching {
                            if let Some(other) = profile.bindings.get(issue.other) {
                                ui.label(format!(
                                    "#{} · {} ({}) · {} · Context: {}",
                                    issue.other + 1,
                                    other.display_name(),
                                    other.action,
                                    other.combination(layout),
                                    other.context.as_deref().unwrap_or("Unknown (not declared)"),
                                ));
                            }
                        }
                    });
                }
            }
            egui::CollapsingHeader::new("Entry properties & source")
                .show(ui, |ui| {
                    ui.label(format!("Source: {}", binding.source_path));
                    ui.small("Properties retain raw values and activation/settings fields. Resolved axes also include retained preset properties; the original export is unchanged in source metadata. Quotes delimit displayed values.");
                                        for inherited in &binding.inherited_properties {
                                            ui.small(format!("Inherited {} from {}", inherited.property.name, inherited.preset_source));
                                        }
                                        if binding.has_nested_data {
                                            ui.small("This entry also contains nested blocks. Inspect them under Import notes & source metadata; exact-duplicate classification is disabled for this entry.");
                                        }
                    if binding.properties.is_empty() {
                        ui.label("Empty override / settings-only entry; no direct properties.");
                    }
                    for property in &binding.properties {
                        ui.monospace(property_text(&property.name, &property.kind, &property.value));
                    }
                });
        });
    });
    open_action
}

fn binding_origin(binding: &wt_keybind_viewer::bindings::model::Binding) -> String {
    let source = match &binding.preset_source {
        Some(path) => format!("Inherited preset: {path}"),
        None => "Source: export".into(),
    };
    if binding.inherited_properties.is_empty() {
        source
    } else {
        format!(
            "{source} · {} retained preset properties (mixed origins)",
            binding.inherited_properties.len()
        )
    }
}

pub fn devices(binding: &wt_keybind_viewer::bindings::model::Binding) -> String {
    let devices: std::collections::BTreeSet<_> =
        binding.inputs.iter().map(|input| input.device()).collect();
    if devices.is_empty() {
        "None explicit (empty override / settings only)".into()
    } else {
        devices.into_iter().collect::<Vec<_>>().join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wt_keybind_viewer::bindings::analyzer;

    #[test]
    fn provenance_distinguishes_export_inheritance_and_mixed_axes() {
        let profile = analyzer::import(include_str!("../../tests/fixtures/inputs_wt.blk")).unwrap();
        let find = |action: &str| {
            profile
                .bindings
                .iter()
                .find(|b| b.action == action)
                .unwrap()
        };
        assert!(
            binding_origin(find("ID_CAMERA_DEFAULT"))
                .contains("Inherited preset: config/hotkeys/hotkey.keyboard_ver1.blk")
        );
        assert_eq!(binding_origin(find("ID_AAM")), "Source: export");
        assert!(binding_origin(find("gm_zoom")).contains("mixed origins"));
    }

    #[test]
    fn both_views_render_at_narrow_and_wide_sizes() {
        let profile = analyzer::import(include_str!("../../tests/fixtures/inputs_wt.blk")).unwrap();
        let analysis = analyzer::analyze(&profile);
        let ctx = egui::Context::default();
        configure(&ctx);
        for width in [480.0, 1280.0] {
            for layout in [Layout::Qwerty, Layout::Azerty] {
                for selected_key in [57, 63, 64, 65] {
                    for keyboard_view in [true, false] {
                        let output = ctx.run(
                            egui::RawInput {
                                screen_rect: Some(egui::Rect::from_min_size(
                                    egui::Pos2::ZERO,
                                    egui::vec2(width, 700.0),
                                )),
                                ..Default::default()
                            },
                            |ctx| {
                                egui::CentralPanel::default().show(ctx, |ui| {
                                    if keyboard_view {
                                        keyboard::show(
                                            ui,
                                            &profile,
                                            &analysis,
                                            layout,
                                            &mut Some(selected_key),
                                        );
                                    } else {
                                        actions::ActionListState::default()
                                            .show(ui, &profile, &analysis, layout);
                                    }
                                });
                            },
                        );
                        assert!(!output.shapes.is_empty());
                    }
                }
            }
        }
    }
}
