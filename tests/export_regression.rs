use wt_keybind_viewer::{
    bindings::{
        analyzer::{analyze, import},
        keymap::{Layout, is_known, key_label},
        model::{Input, IssueKind, Section},
    },
    blk,
};

const EXPORT: &str = include_str!("fixtures/inputs_wt.blk");

#[test]
fn real_export_snapshot_preserves_every_explicit_entry() {
    let p = import(EXPORT).unwrap();
    let a = analyze(&p);
    assert_eq!(p.bindings.len(), 97);
    assert_eq!(
        p.bindings
            .iter()
            .filter(|b| b.section == Section::Hotkeys)
            .count(),
        92
    );
    assert_eq!(
        p.bindings
            .iter()
            .filter(|b| b.section == Section::Axes)
            .count(),
        5
    );
    assert_eq!(a.by_key.len(), 46);
    assert_eq!(
        p.bindings
            .iter()
            .flat_map(|b| &b.inputs)
            .filter(|i| matches!(i, Input::Keyboard(_)))
            .count(),
        80
    );
    assert!(a.by_key.keys().all(|&code| is_known(code)));
    assert!(!a.overlap_checks_truncated);
    assert_eq!(p.preset_paths, ["config/hotkeys/hotkey.keyboard_ver1.blk"]);
    assert!(p.bindings.iter().all(|b| b.context.is_none()));
    assert!(p.warnings.iter().any(|w| w.contains("not resolved")));
}

#[test]
fn english_names_cover_known_export_entries_without_guessing_missing_labels() {
    let profile = import(EXPORT).unwrap();
    assert_eq!(
        profile
            .bindings
            .iter()
            .filter(|b| b.english_name().is_some())
            .count(),
        81
    );
    assert!(
        profile
            .bindings
            .iter()
            .filter(|b| b.section == Section::Axes)
            .all(|b| b.english_name().is_some())
    );
    let missing: std::collections::BTreeSet<_> = profile
        .bindings
        .iter()
        .filter(|b| b.english_name().is_none())
        .map(|b| b.action.as_str())
        .collect();
    assert_eq!(
        missing.into_iter().collect::<Vec<_>>(),
        [
            "human_uav_climb_rangeMax",
            "human_uav_climb_rangeMin",
            "human_uav_pitch_rangeMax",
            "human_uav_pitch_rangeMin",
            "human_uav_roll_rangeMax",
            "human_uav_roll_rangeMin",
            "human_uav_throttle_rangeMax",
            "human_uav_throttle_rangeMin",
            "human_uav_yaw_rangeMax",
            "human_uav_yaw_rangeMin",
        ]
    );
    let shots: Vec<_> = profile
        .bindings
        .iter()
        .filter(|b| b.action == "ID_SCREENSHOT")
        .collect();
    assert_eq!(shots.len(), 3);
    assert!(shots.iter().all(|b| b.display_name() == "Screenshot"));
}

#[test]
fn screenshot_duplicates_and_modifier_combo_are_not_overwritten() {
    let p = import(EXPORT).unwrap();
    let a = analyze(&p);
    let shots: Vec<_> = p
        .bindings
        .iter()
        .enumerate()
        .filter(|(_, b)| b.action == "ID_SCREENSHOT")
        .collect();
    assert_eq!(shots.len(), 3);
    assert_eq!(
        shots[0].1.inputs,
        [Input::Keyboard(42), Input::Keyboard(183)]
    );
    assert_eq!(shots[1].1.inputs, [Input::Keyboard(183)]);
    assert_eq!(shots[2].1.inputs, [Input::Keyboard(183)]);
    assert!(a.issues[shots[0].0].is_empty());
    assert!(
        a.issues[shots[1].0]
            .iter()
            .any(|issue| issue.kind == IssueKind::Duplicate && issue.other == shots[2].0)
    );
}

#[test]
fn space_reuse_has_unknown_context_not_confirmed_conflicts() {
    let p = import(EXPORT).unwrap();
    let a = analyze(&p);
    let actions: Vec<_> = a
        .indices_for_key(57)
        .iter()
        .map(|&i| p.bindings[i].action.as_str())
        .collect();
    assert_eq!(
        actions,
        [
            "ID_AAM",
            "ID_BOMBS",
            "ID_CONTINUE",
            "ID_CONTINUE_SETUP",
            "ID_ROCKETS"
        ]
    );
    for &index in a.indices_for_key(57) {
        assert_eq!(a.issues[index].len(), 4);
        assert!(
            a.issues[index]
                .iter()
                .all(|i| i.kind == IssueKind::ContextUnknown)
        );
    }
    assert_eq!(key_label(57, Layout::Qwerty), "Space");
    assert_eq!(key_label(57, Layout::Azerty), "Space");
}

#[test]
fn empty_entries_axes_and_device_metadata_remain_available() {
    let p = import(EXPORT).unwrap();
    assert!(
        p.bindings
            .iter()
            .any(|b| b.action == "ID_CAMERA_FPS" && b.inputs.is_empty())
    );
    assert!(p.bindings.iter().any(|b| b.action == "gm_zoom"
        && b.inputs
            == [Input::Axis {
                kind: "mouseAxisId".into(),
                code: -1
            }]));
    let controls = p.document.blocks().find(|b| b.name == "controls").unwrap();
    let devices = controls
        .blocks()
        .find(|b| b.name == "deviceMapping")
        .unwrap();
    let joysticks: Vec<_> = devices.blocks().collect();
    assert_eq!(joysticks.len(), 3);
    assert_eq!(joysticks[0].property("buttonsOffset").unwrap().value, "0");
    assert_eq!(joysticks[1].property("buttonsOffset").unwrap().value, "28");
    assert!(
        p.bindings
            .iter()
            .any(|b| b.inputs.contains(&Input::GamepadButton(40)))
    );
}

#[test]
fn importing_never_changes_the_original_fixture() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/inputs_wt.blk");
    let before = std::fs::read(path).unwrap();
    let profile = import(std::str::from_utf8(&before).unwrap()).unwrap();
    let _analysis = analyze(&profile);
    for binding in &profile.bindings {
        let _name = binding.display_name();
    }
    assert_eq!(std::fs::read(path).unwrap(), before);
}

#[test]
fn malformed_fixture_reports_the_actual_location() {
    let mut broken = EXPORT.to_owned();
    broken.push_str("\nbad:i=not_an_integer");
    let error = blk::parse(&broken).unwrap_err();
    assert_eq!(error.line, EXPORT.lines().count() + 2);
    assert!(error.column > 1);
    assert!(error.message.contains("32-bit integer"));
}
