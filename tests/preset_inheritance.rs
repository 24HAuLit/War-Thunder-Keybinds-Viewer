use wt_keybind_viewer::{
    bindings::{
        analyzer::{analyze, import, import_explicit},
        model::{Binding, Input, IssueKind, Profile, Section},
    },
    blk,
};

const EXPORT: &str = include_str!("fixtures/inputs_wt.blk");
const KEYBOARD: &str = "config/hotkeys/hotkey.keyboard_ver1.blk";
const BASE: &str = "config/hotkeys/hotkey.base.keyboard.blk";

fn with_base(entries: &str) -> String {
    format!("controls {{version:i=5; basePresetPaths {{default:t=\"{KEYBOARD}\"}} {entries}}}")
}

fn entries<'a>(profile: &'a Profile, action: &str, section: Section) -> Vec<&'a Binding> {
    profile
        .bindings
        .iter()
        .filter(|b| b.action == action && b.section == section)
        .collect()
}

#[test]
fn resolves_complete_chain_base_first_and_preserves_original_document() {
    let profile = import(EXPORT).unwrap();
    assert_eq!(profile.document, blk::parse(EXPORT).unwrap());
    assert_eq!(profile.preset_paths, [KEYBOARD]);
    assert_eq!(
        profile.resolved_presets,
        [
            "config/hotkeys/hotkey.event.human.blk",
            "config/hotkeys/hotkey.empty_ver1.blk",
            BASE,
            "config/hotkeys/hotkey.base.keyboard_simulator.blk",
            KEYBOARD,
        ]
    );
    for (action, code) in [
        ("ID_CAMERA_DEFAULT", 63),
        ("ID_CAMERA_GUNNER", 64),
        ("ID_CAMERA_BOMBVIEW", 65),
    ] {
        let binding = entries(&profile, action, Section::Hotkeys);
        assert_eq!(binding.len(), 1);
        assert_eq!(binding[0].inputs, [Input::Keyboard(code)]);
        assert_eq!(binding[0].preset_source.as_deref(), Some(KEYBOARD));
        assert_eq!(
            binding[0].source_path,
            format!("controls / hotkeys / {action}")
        );
        assert!(binding[0].context.is_none());
    }
    for text in [
        "aa326f10",
        "game-version",
        "fallbacks",
        "implicit empty",
        "not proven free",
    ] {
        assert!(
            profile.warnings.iter().any(|w| w.contains(text)),
            "missing warning: {text}"
        );
    }
}

#[test]
fn explicit_api_retains_original_counts_order_duplicates_and_metadata() {
    let explicit = import_explicit(EXPORT).unwrap();
    assert_eq!(explicit.bindings.len(), 97);
    assert_eq!(
        explicit
            .bindings
            .iter()
            .filter(|b| b.section == Section::Hotkeys)
            .count(),
        92
    );
    assert_eq!(
        explicit
            .bindings
            .iter()
            .filter(|b| b.section == Section::Axes)
            .count(),
        5
    );
    assert_eq!(analyze(&explicit).by_key.len(), 46);
    assert_eq!(
        explicit
            .bindings
            .iter()
            .flat_map(|b| &b.inputs)
            .filter(|i| matches!(i, Input::Keyboard(_)))
            .count(),
        80
    );
    assert!(explicit.resolved_presets.is_empty());
    assert!(
        explicit
            .bindings
            .iter()
            .all(|b| b.preset_source.is_none() && b.inherited_properties.is_empty())
    );
    assert!(explicit.warnings.iter().any(|w| w.contains("not resolved")));

    let resolved = import(EXPORT).unwrap();
    let original_hotkeys: Vec<_> = explicit
        .bindings
        .iter()
        .filter(|b| b.section == Section::Hotkeys)
        .collect();
    let resolved_explicit: Vec<_> = resolved
        .bindings
        .iter()
        .filter(|b| b.section == Section::Hotkeys && b.preset_source.is_none())
        .collect();
    assert_eq!(original_hotkeys.len(), resolved_explicit.len());
    for (original, effective) in original_hotkeys.into_iter().zip(resolved_explicit) {
        assert_eq!(original.properties, effective.properties);
        assert_eq!(original.inputs, effective.inputs);
        assert_eq!(original.source_path, effective.source_path);
    }
}

#[test]
fn hotkey_override_resets_all_inherited_alternatives_without_deduplicating_export() {
    let profile = import(EXPORT).unwrap();
    let analysis = analyze(&profile);
    for action in ["ID_CAMERA_FPS", "ID_CAMERA_FPS_HELICOPTER"] {
        let bindings = entries(&profile, action, Section::Hotkeys);
        assert_eq!(bindings.len(), 1);
        assert!(bindings[0].inputs.is_empty());
        assert!(bindings[0].preset_source.is_none());
    }
    assert!(analysis.indices_for_key(60).iter().all(|&i| !matches!(
        profile.bindings[i].action.as_str(),
        "ID_CAMERA_FPS" | "ID_CAMERA_FPS_HELICOPTER"
    )));
    let aam = entries(&profile, "ID_AAM", Section::Hotkeys);
    assert_eq!(aam.len(), 1);
    assert_eq!(aam[0].inputs, [Input::Keyboard(57)]);
    assert!(aam[0].preset_source.is_none());

    let shots: Vec<_> = profile
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
    assert!(shots.iter().all(|(_, b)| b.preset_source.is_none()));
    assert!(
        analysis.issues[shots[1].0]
            .iter()
            .any(|i| i.kind == IssueKind::Duplicate && i.other == shots[2].0)
    );
}

#[test]
fn empty_sibling_does_not_cancel_nonempty_alternatives_in_either_order() {
    for siblings in [
        "ID_CAMERA_DEFAULT{} ID_CAMERA_DEFAULT{keyboardKey:i=7} ID_CAMERA_DEFAULT{keyboardKey:i=8}",
        "ID_CAMERA_DEFAULT{keyboardKey:i=7} ID_CAMERA_DEFAULT{} ID_CAMERA_DEFAULT{keyboardKey:i=8}",
        "ID_CAMERA_DEFAULT{keyboardKey:i=7} ID_CAMERA_DEFAULT{keyboardKey:i=8} ID_CAMERA_DEFAULT{}",
    ] {
        let profile = import(&with_base(&format!("hotkeys {{{siblings}}}"))).unwrap();
        let bindings = entries(&profile, "ID_CAMERA_DEFAULT", Section::Hotkeys);
        assert_eq!(bindings.len(), 3);
        assert_eq!(bindings.iter().filter(|b| b.inputs.is_empty()).count(), 1);
        assert!(bindings.iter().all(|b| b.preset_source.is_none()));
        for code in [7, 8] {
            assert!(bindings.iter().any(|b| b.inputs == [Input::Keyboard(code)]));
        }
        assert!(
            !bindings
                .iter()
                .any(|b| b.inputs.contains(&Input::Keyboard(63)))
        );
    }
}

#[test]
fn axes_overlay_scalars_preserve_disabled_values_and_record_property_origins() {
    let profile = import(EXPORT).unwrap();
    let zoom = entries(&profile, "gm_zoom", Section::Axes);
    assert_eq!(zoom.len(), 1);
    assert_eq!(
        zoom[0].inputs,
        [Input::Axis {
            kind: "mouseAxisId".into(),
            code: -1
        }]
    );
    assert!(zoom[0].preset_source.is_none());
    assert!(
        zoom[0]
            .properties
            .iter()
            .any(|p| p.name == "relative" && p.value == "yes")
    );
    assert_eq!(zoom[0].inherited_properties.len(), 1);
    assert_eq!(zoom[0].inherited_properties[0].property.name, "relative");
    assert_eq!(zoom[0].inherited_properties[0].preset_source, BASE);

    let throttle = entries(&profile, "human_uav_throttle", Section::Axes);
    assert_eq!(throttle.len(), 1);
    assert_eq!(
        throttle[0].inputs,
        [Input::Axis {
            kind: "mouseAxisId".into(),
            code: 7
        }]
    );
    assert!(
        throttle[0]
            .inherited_properties
            .iter()
            .any(|p| p.property.name == "relative"
                && p.property.value == "yes"
                && p.preset_source == BASE)
    );
    assert!(
        !throttle[0]
            .inherited_properties
            .iter()
            .any(|p| p.property.name == "mouseAxisId")
    );

    let profile = import(&with_base(
        "axes {gm_zoom {nonlinearity:r=2} human_uav_throttle {innerDeadzone:r=0.1}}",
    ))
    .unwrap();
    let zoom = entries(&profile, "gm_zoom", Section::Axes)[0];
    assert_eq!(
        zoom.inputs,
        [Input::Axis {
            kind: "mouseAxisId".into(),
            code: 3
        }]
    );
    assert!(
        zoom.inherited_properties
            .iter()
            .any(|p| p.property.name == "mouseAxisId" && p.preset_source == BASE)
    );
    let throttle = entries(&profile, "human_uav_throttle", Section::Axes)[0];
    assert!(
        throttle
            .properties
            .iter()
            .any(|p| p.name == "relative" && p.value == "yes")
    );
    assert!(
        throttle.inputs.is_empty(),
        "do not invent game fallback inputs"
    );
}

#[test]
fn empty_and_repeated_axes_retain_prior_scalars_without_false_exact_duplicates() {
    let profile = import(&with_base(
        "axes {gm_zoom{} gm_zoom{} gm_zoom{relative:b=no} gm_zoom{}}",
    ))
    .unwrap();
    let zoom = entries(&profile, "gm_zoom", Section::Axes);
    assert_eq!(zoom.len(), 4);
    for (index, binding) in zoom.iter().enumerate() {
        assert_eq!(
            binding.inputs,
            [Input::Axis {
                kind: "mouseAxisId".into(),
                code: 3
            }]
        );
        assert!(binding.preset_source.is_none());
        assert!(
            binding
                .properties
                .iter()
                .any(|p| p.name == "relative" && p.value == if index < 2 { "yes" } else { "no" })
        );
    }
    assert!(
        !zoom[3]
            .inherited_properties
            .iter()
            .any(|p| p.property.name == "relative")
    );
    let analysis = analyze(&profile);
    for (index, binding) in profile
        .bindings
        .iter()
        .enumerate()
        .filter(|(_, b)| b.action == "gm_zoom")
    {
        assert!(!binding.inherited_properties.is_empty());
        assert!(
            analysis.issues[index]
                .iter()
                .all(|issue| issue.kind != IssueKind::Duplicate)
        );
    }
}

#[test]
fn structural_context_and_section_are_part_of_override_identity() {
    let profile = import(&with_base("hotkeys {air {ID_CAMERA_DEFAULT{keyboardKey:i=8}}} axes {ID_CAMERA_DEFAULT{mouseAxisId:i=1} group {gm_zoom{mouseAxisId:i=-1}}}")).unwrap();
    let camera = entries(&profile, "ID_CAMERA_DEFAULT", Section::Hotkeys);
    assert_eq!(camera.len(), 2);
    assert!(
        camera
            .iter()
            .any(|b| b.context.is_none() && b.inputs == [Input::Keyboard(63)])
    );
    assert!(camera.iter().any(|b| b.context.as_deref() == Some("air")
        && b.source_path == "controls / hotkeys / air / ID_CAMERA_DEFAULT"));
    let zoom = entries(&profile, "gm_zoom", Section::Axes);
    assert_eq!(zoom.len(), 2);
    assert!(
        zoom.iter()
            .any(|b| b.context.is_none() && b.preset_source.as_deref() == Some(BASE))
    );
    assert!(
        zoom.iter()
            .any(|b| b.context.as_deref() == Some("group") && b.inherited_properties.is_empty())
    );
}

#[test]
fn unknown_and_traversal_references_are_not_normalized_or_followed() {
    for path in [
        "config/hotkeys/missing.blk",
        "config/hotkeys/../hotkeys/hotkey.keyboard_ver1.blk",
        "./config/hotkeys/hotkey.keyboard_ver1.blk",
        "../../tests/fixtures/inputs_wt.blk",
        "/etc/passwd",
        "https://example.invalid/preset.blk",
        "wt/config/hotkeys/hotkey.keyboard_ver1.blk",
    ] {
        let source = format!(
            "controls {{version:i=5; basePresetPaths {{default:t=\"{path}\"}} hotkeys {{A{{keyboardKey:i=7}} A{{}}}}}}"
        );
        let profile = import(&source).unwrap();
        assert_eq!(profile.bindings.len(), 2);
        assert!(profile.resolved_presets.is_empty());
        assert!(profile.bindings.iter().all(|b| b.preset_source.is_none()));
        assert_eq!(profile.document, blk::parse(&source).unwrap());
        assert!(
            profile
                .warnings
                .iter()
                .any(|w| w.contains(path) && w.contains("not in the exact embedded allowlist"))
        );
        assert!(
            profile
                .warnings
                .iter()
                .any(|w| w.contains("explicit export entries are retained"))
        );
    }
}

#[test]
fn includes_nondefault_and_nested_references_never_merge() {
    let source = format!(
        "include \"{KEYBOARD}\"; controls {{version:i=5; basePresetPaths {{air:t=\"{KEYBOARD}\"; nested {{default:t=\"{KEYBOARD}\"}}}} hotkeys {{A{{}}}}}}"
    );
    let profile = import(&source).unwrap();
    assert_eq!(profile.bindings.len(), 1);
    assert_eq!(profile.includes, [KEYBOARD]);
    assert!(profile.resolved_presets.is_empty());
    assert!(
        profile
            .warnings
            .iter()
            .any(|w| w.contains("Ignored non-default"))
    );
    assert!(
        profile
            .warnings
            .iter()
            .any(|w| w.contains("Ignored nested"))
    );
    assert!(
        profile
            .warnings
            .iter()
            .any(|w| w.contains("never followed"))
    );

    let profile = import(&with_base(&format!("basePresetPaths {{air:t=\"{BASE}\"}}"))).unwrap();
    assert_eq!(profile.resolved_presets.len(), 5);
    assert!(
        profile
            .warnings
            .iter()
            .any(|w| w.contains("Ignored non-default"))
    );
}

#[test]
fn resolution_is_version_five_only_without_guessed_legacy_rules() {
    for version in ["", "version:i=4;", "version:i=6;", "version:t=5;"] {
        let source = format!(
            "controls {{{version} basePresetPaths {{default:t=\"{KEYBOARD}\"}} hotkeys {{A{{}}}}}}"
        );
        let profile = import(&source).unwrap();
        assert_eq!(profile.bindings.len(), 1);
        assert!(profile.resolved_presets.is_empty());
        assert!(
            profile
                .warnings
                .iter()
                .any(|w| w.contains("only controls version 5"))
        );
    }
}

#[test]
fn no_base_sample_remains_explicit_without_implicit_empty_preset() {
    let source = "include \"never-follow.blk\"; controls {version:i=5; hotkeys {A{} A{keyboardKey:i=7} A{keyboardKey:i=7}} axes {gm_zoom{} gm_zoom{mouseAxisId:i=-1}}}";
    let resolved = import(source).unwrap();
    let explicit = import_explicit(source).unwrap();
    assert_eq!(resolved.bindings.len(), 5);
    assert_eq!(resolved.document, explicit.document);
    assert!(resolved.resolved_presets.is_empty());
    for (a, b) in resolved.bindings.iter().zip(&explicit.bindings) {
        assert_eq!(a.properties, b.properties);
        assert_eq!(a.inputs, b.inputs);
        assert_eq!(a.source_path, b.source_path);
        assert!(a.preset_source.is_none() && a.inherited_properties.is_empty());
    }
    assert_eq!(analyze(&resolved).by_key, analyze(&explicit).by_key);
}
