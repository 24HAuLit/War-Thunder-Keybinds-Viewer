use eframe::egui;
use wt_keybind_viewer::bindings::{
    analyzer::Analysis,
    keymap::Layout,
    model::{Binding, IssueKind, Profile, Section},
};

use super::{binding_card, devices};

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum IssueFilter {
    #[default]
    All,
    Any,
    None,
    Kind(IssueKind),
}

impl IssueFilter {
    fn label(self) -> &'static str {
        match self {
            Self::All => "All entries",
            Self::Any => "With analysis notes",
            Self::None => "Without analysis notes",
            Self::Kind(kind) => kind.label(),
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum EmptyFilter {
    #[default]
    All,
    Only,
    Exclude,
}

impl EmptyFilter {
    fn label(self) -> &'static str {
        match self {
            Self::All => "Include empty overrides",
            Self::Only => "Only empty / settings-only",
            Self::Exclude => "Only entries with inputs",
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum SortBy {
    #[default]
    Action,
    Combination,
}

impl SortBy {
    fn label(self) -> &'static str {
        match self {
            Self::Action => "Action ID",
            Self::Combination => "Combination",
        }
    }
}

#[derive(Default)]
pub struct ActionListState {
    query: String,
    device: Option<String>,
    section: Option<Section>,
    issues: IssueFilter,
    empty: EmptyFilter,
    sort: SortBy,
    descending: bool,
}

impl ActionListState {
    pub fn focus_action(&mut self, action: String) {
        *self = Self {
            query: action,
            ..Default::default()
        };
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        profile: &Profile,
        analysis: &Analysis,
        layout: Layout,
    ) {
        // Keep filters inside the vertical viewport as well, so very short or
        // narrow windows cannot push the entire result list outside the panel.
        egui::ScrollArea::vertical()
            .id_salt("action_list")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.query)
                        .hint_text("Search action ID, English action name, combo, device, context, or preset…")
                        .desired_width(ui.available_width()),
                );
                ui.horizontal_wrapped(|ui| {
                    ui.label("Device:");
                    egui::ComboBox::from_id_salt("device_filter")
                        .selected_text(self.device.as_deref().unwrap_or("All devices"))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.device, None, "All devices");
                            for device in ["Keyboard", "Mouse", "Gamepad", "Gamepad / axis", "Unknown", "None explicit"] {
                                ui.selectable_value(&mut self.device, Some(device.into()), device);
                            }
                        });
                    ui.label("Section:");
                    egui::ComboBox::from_id_salt("section_filter")
                        .selected_text(self.section.map_or("All sections", Section::name))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.section, None, "All sections");
                            for section in [Section::Hotkeys, Section::Axes] {
                                ui.selectable_value(&mut self.section, Some(section), section.name());
                            }
                        });
                    if ui.button("Reset filters").clicked() {
                        *self = Self::default();
                    }
                });
                ui.horizontal_wrapped(|ui| {
                    ui.label("Analysis:");
                    egui::ComboBox::from_id_salt("issues_filter")
                        .selected_text(self.issues.label())
                        .show_ui(ui, |ui| {
                            for filter in [
                                IssueFilter::All,
                                IssueFilter::Any,
                                IssueFilter::None,
                                IssueFilter::Kind(IssueKind::Duplicate),
                                IssueFilter::Kind(IssueKind::PotentialConflict),
                                IssueFilter::Kind(IssueKind::ContextUnknown),
                            ] {
                                ui.selectable_value(&mut self.issues, filter, filter.label());
                            }
                        });
                    egui::ComboBox::from_id_salt("empty_filter")
                        .selected_text(self.empty.label())
                        .show_ui(ui, |ui| {
                            for filter in [EmptyFilter::All, EmptyFilter::Only, EmptyFilter::Exclude] {
                                ui.selectable_value(&mut self.empty, filter, filter.label());
                            }
                        });
                });
                ui.horizontal_wrapped(|ui| {
                    ui.label("Sort:");
                    egui::ComboBox::from_id_salt("action_sort")
                        .selected_text(self.sort.label())
                        .show_ui(ui, |ui| {
                            for sort in [SortBy::Action, SortBy::Combination] {
                                ui.selectable_value(&mut self.sort, sort, sort.label());
                            }
                        });
                    ui.selectable_value(&mut self.descending, false, "Ascending");
                    ui.selectable_value(&mut self.descending, true, "Descending");
                });
                ui.small("English action names use official game labels where known; unknown IDs use human-readable fallbacks. Export entries override inherited hotkeys; surviving repeated entries and exact duplicates remain separate. Axis properties may have mixed origins. Unknown context is never inferred from an ID.");
                let indices = self.filtered_indices(profile, analysis, layout);
                ui.label(format!("{} of {} entries", indices.len(), profile.bindings.len()));
                ui.separator();
                if indices.is_empty() {
                    ui.label("No entries match these filters. Try a broader search or reset the filters.");
                }
                for index in indices {
                    binding_card(ui, profile, analysis, index, layout, false);
                    ui.add_space(3.0);
                }
            });
    }

    fn filtered_indices(
        &self,
        profile: &Profile,
        analysis: &Analysis,
        layout: Layout,
    ) -> Vec<usize> {
        let query = self.query.trim().to_lowercase();
        // Cache sort strings for this pass, rather than formatting combinations
        // again for every comparison. Original indices provide stable tie order.
        let mut entries: Vec<_> = profile
            .bindings
            .iter()
            .enumerate()
            .filter(|(index, binding)| self.matches(binding, *index, analysis, layout, &query))
            .map(|(index, binding)| {
                let key = match self.sort {
                    SortBy::Action => binding.action.to_lowercase(),
                    SortBy::Combination => binding.combination(layout).to_lowercase(),
                };
                (index, key)
            })
            .collect();
        entries.sort_by(|(left_index, left), (right_index, right)| {
            let order = left.cmp(right);
            let order = if self.descending {
                order.reverse()
            } else {
                order
            };
            order.then(left_index.cmp(right_index))
        });
        entries.into_iter().map(|(index, _)| index).collect()
    }

    fn matches(
        &self,
        binding: &Binding,
        index: usize,
        analysis: &Analysis,
        layout: Layout,
        query: &str,
    ) -> bool {
        if self
            .section
            .is_some_and(|section| section != binding.section)
        {
            return false;
        }
        if let Some(device) = &self.device {
            let matches = if device == "None explicit" {
                binding.inputs.is_empty()
            } else {
                binding.inputs.iter().any(|input| input.device() == device)
            };
            if !matches {
                return false;
            }
        }
        match self.empty {
            EmptyFilter::Only if !binding.inputs.is_empty() => return false,
            EmptyFilter::Exclude if binding.inputs.is_empty() => return false,
            _ => {}
        }
        let issues = analysis.issues.get(index).map(Vec::as_slice).unwrap_or(&[]);
        let issue_match = match self.issues {
            IssueFilter::All => true,
            IssueFilter::Any => !issues.is_empty(),
            IssueFilter::None => issues.is_empty(),
            IssueFilter::Kind(kind) => issues.iter().any(|issue| issue.kind == kind),
        };
        if !issue_match {
            return false;
        }
        if query.is_empty() {
            return true;
        }
        format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
            binding.action,
            binding.display_name(),
            binding.combination(layout),
            devices(binding),
            binding
                .context
                .as_deref()
                .unwrap_or("Unknown (not declared)"),
            binding.section.name(),
            binding.source_path,
            super::binding_origin(binding),
        )
        .to_lowercase()
        .contains(query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wt_keybind_viewer::bindings::analyzer;

    fn fixture() -> (Profile, Analysis) {
        let profile = analyzer::import(
            "controls { hotkeys { ID_TEST { keyboardKey:i=16 } ID_TEST { mouseButton:i=0 } ID_EMPTY {} ID_TEST { keyboardKey:i=16 } } }",
        ).unwrap();
        let analysis = analyzer::analyze(&profile);
        (profile, analysis)
    }

    #[test]
    fn repeated_ids_and_duplicates_remain_separate() {
        let (profile, analysis) = fixture();
        let mut state = ActionListState::default();
        state.focus_action("ID_TEST".into());
        assert_eq!(
            state.filtered_indices(&profile, &analysis, Layout::Qwerty),
            vec![0, 1, 3]
        );
        state.descending = true;
        assert_eq!(
            state.filtered_indices(&profile, &analysis, Layout::Qwerty),
            vec![0, 1, 3]
        );
    }

    #[test]
    fn device_empty_and_section_filters() {
        let (profile, analysis) = fixture();
        let mut state = ActionListState {
            device: Some("Mouse".into()),
            ..Default::default()
        };
        assert_eq!(
            state.filtered_indices(&profile, &analysis, Layout::Qwerty),
            vec![1]
        );
        state.device = None;
        state.empty = EmptyFilter::Only;
        assert_eq!(
            state.filtered_indices(&profile, &analysis, Layout::Qwerty),
            vec![2]
        );
        state.section = Some(Section::Axes);
        assert!(
            state
                .filtered_indices(&profile, &analysis, Layout::Qwerty)
                .is_empty()
        );
    }

    #[test]
    fn search_covers_english_name_and_original_id_case_insensitively() {
        let profile = analyzer::import(
            "controls { hotkeys { ID_AAM { keyboardKey:i=16 } ID_TEST { mouseButton:i=0 } } }",
        )
        .unwrap();
        let analysis = analyzer::analyze(&profile);
        for query in [
            "Fire air-to-air missile",
            "fIrE AiR-tO-aIr MiSsIlE",
            "ID_AAM",
            "id_aam",
        ] {
            let state = ActionListState {
                query: query.into(),
                ..Default::default()
            };
            assert_eq!(
                state.filtered_indices(&profile, &analysis, Layout::Qwerty),
                vec![0],
                "{query}"
            );
        }
    }

    #[test]
    fn search_covers_fallback_device_context_and_layout_legends() {
        let (profile, analysis) = fixture();
        for (query, layout, expected) in [
            ("test", Layout::Qwerty, vec![0, 1, 3]),
            ("mouse button 0", Layout::Qwerty, vec![1]),
            ("keyboard", Layout::Qwerty, vec![0, 3]),
            ("q", Layout::Qwerty, vec![0, 3]),
            ("q", Layout::Azerty, vec![]),
            ("unknown", Layout::Qwerty, vec![2, 0, 1, 3]),
            ("no explicit input", Layout::Azerty, vec![2]),
        ] {
            let state = ActionListState {
                query: query.into(),
                ..Default::default()
            };
            assert_eq!(
                state.filtered_indices(&profile, &analysis, layout),
                expected,
                "{query}"
            );
        }
    }

    #[test]
    fn combination_sort_reverses_values_but_keeps_source_order_for_ties() {
        let (profile, analysis) = fixture();
        let mut state = ActionListState {
            sort: SortBy::Combination,
            ..Default::default()
        };
        assert_eq!(
            state.filtered_indices(&profile, &analysis, Layout::Qwerty),
            vec![1, 2, 0, 3]
        );
        state.descending = true;
        assert_eq!(
            state.filtered_indices(&profile, &analysis, Layout::Qwerty),
            vec![0, 3, 2, 1]
        );
    }

    #[test]
    fn issue_filter_uses_per_entry_analysis() {
        let (profile, analysis) = fixture();
        let state = ActionListState {
            issues: IssueFilter::Kind(IssueKind::Duplicate),
            ..Default::default()
        };
        assert_eq!(
            state.filtered_indices(&profile, &analysis, Layout::Qwerty),
            vec![0, 3]
        );
    }
}
