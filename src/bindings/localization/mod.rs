//! Pinned game-English labels, separate from the identifiers used by the parser.

use super::model::Section;

mod english;

pub fn english_name(action: &str, section: Section) -> Option<&'static str> {
    let namespace = match section {
        Section::Hotkeys => "hotkeys",
        Section::Axes => "controls",
    };
    let key = format!("{namespace}/{action}");
    english::ENGLISH
        .binary_search_by_key(&key.as_str(), |&(key, _)| key)
        .ok()
        .map(|index| english::ENGLISH[index].1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bindings::analyzer::import;

    #[test]
    fn catalogue_is_sorted_unique_and_nonempty() {
        assert!(english::ENGLISH.len() > 1500);
        for &(key, label) in english::ENGLISH {
            assert!(key.starts_with("hotkeys/") || key.starts_with("controls/"));
            assert!(!label.trim().is_empty(), "{key}");
        }
        for pair in english::ENGLISH.windows(2) {
            assert!(
                pair[0].0 < pair[1].0,
                "Unsorted or duplicate key: {}",
                pair[1].0
            );
        }
    }

    #[test]
    fn uses_exact_ids_and_section_specific_game_labels() {
        assert_eq!(
            english_name("ID_FIRE", Section::Hotkeys),
            Some("Fire guns/cannons")
        );
        assert_eq!(
            english_name("ID_AAM", Section::Hotkeys),
            Some("Fire air-to-air missile")
        );
        assert_eq!(
            english_name("ID_SCREENSHOT", Section::Hotkeys),
            Some("Screenshot")
        );
        assert_eq!(
            english_name("gm_zoom", Section::Axes),
            Some("Zoom axis (Ground)")
        );
        assert_eq!(
            english_name("gm_zoom", Section::Hotkeys),
            Some("Enable ground vehicle zoom axis")
        );
        assert_eq!(
            english_name("elevator_rangeMax", Section::Hotkeys),
            Some("Pitch up")
        );
        assert_eq!(
            english_name("human_uav_pitch", Section::Axes),
            Some("Pitch axis")
        );
        assert_eq!(
            english_name("ID_CONTROL_MODE_HUMAN_UAV", Section::Hotkeys),
            Some("Hover mode")
        );
        assert_eq!(english_name("id_fire", Section::Hotkeys), None);
        assert_eq!(english_name("ID_FIRE", Section::Axes), None);
        assert_eq!(
            english_name("ID_NOT_A_REAL_COMMAND", Section::Hotkeys),
            None
        );
    }

    #[test]
    fn binding_names_do_not_change_identifiers_or_source_data() {
        let profile = import("controls { hotkeys { ID_AAM { keyboardKey:i=57 } ID_FUTURE_COMMAND {} future_axis_rangeMax {} } }").unwrap();
        let binding = &profile.bindings[0];
        let document_before = format!("{:?}", profile.document);
        assert_eq!(binding.english_name(), Some("Fire air-to-air missile"));
        assert_eq!(binding.display_name(), "Fire air-to-air missile");
        assert_eq!(binding.action, "ID_AAM");
        assert_eq!(profile.bindings[1].english_name(), None);
        assert_eq!(profile.bindings[1].display_name(), "future command");
        assert_eq!(profile.bindings[2].english_name(), None);
        assert_eq!(profile.bindings[2].display_name(), "future axis rangemax");
        assert_eq!(format!("{:?}", profile.document), document_before);
    }
}
