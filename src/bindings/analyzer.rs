use std::collections::{BTreeMap, BTreeSet};

use thiserror::Error;

use crate::blk::{self, Block, Property};

use super::model::{Binding, BindingIssue, InheritedProperty, Input, IssueKind, Profile, Section};
use super::presets;

#[derive(Debug, Error)]
pub enum ImportError {
    #[error(transparent)]
    Parse(#[from] blk::ParseError),
    #[error("No controls block found; select a text .blk controls export from War Thunder")]
    MissingControls,
    #[error(
        "Controls structure exceeds the 64 MiB derived-path budget; deeply grouped or unusually long names are not supported"
    )]
    StructureTooLarge,
}

/// Resolve exact embedded version-5 default presets, without filesystem/network I/O.
pub fn import(source: &str) -> Result<Profile, ImportError> {
    import_with_mode(source, true)
}

/// Import only explicit entries, preserving the original export/count behavior.
pub fn import_explicit(source: &str) -> Result<Profile, ImportError> {
    import_with_mode(source, false)
}

fn import_with_mode(source: &str, resolve: bool) -> Result<Profile, ImportError> {
    let document = blk::parse(source)?;
    let controls: Vec<_> = document.blocks().filter(|b| b.name == "controls").collect();
    if controls.is_empty() {
        return Err(ImportError::MissingControls);
    }
    let mut bindings = Vec::new();
    let mut preset_paths = Vec::new();
    let mut resolved_presets = Vec::new();
    let mut warnings = Vec::new();
    let mut path_budget = 64 * 1024 * 1024;
    for control in controls {
        for block in control.blocks().filter(|b| b.name == "basePresetPaths") {
            collect_values(block, &mut preset_paths);
        }
        let layers = if resolve {
            presets::resolve(control, &mut warnings)
        } else {
            Vec::new()
        };
        let mut effective = Vec::new();
        for layer in &layers {
            let mut entries = collect_control(layer.controls, &mut path_budget)?;
            for binding in &mut entries {
                charge(&mut path_budget, layer.path.len())?;
                binding.preset_source = Some(layer.path.clone());
            }
            merge_layer(&mut effective, entries, &mut path_budget)?;
            if !resolved_presets.contains(&layer.path) {
                resolved_presets.push(layer.path.clone());
            }
        }
        let explicit = collect_control(control, &mut path_budget)?;
        if layers.is_empty() {
            // In particular, no-base files do not acquire an implicit empty preset
            // and repeated explicit entries are never normalized away.
            effective = explicit;
        } else {
            merge_layer(&mut effective, explicit, &mut path_budget)?;
        }
        bindings.extend(effective);
    }
    let mut includes = Vec::new();
    collect_includes(&document, &mut includes);
    if !resolve && (!preset_paths.is_empty() || !includes.is_empty()) {
        warnings.push("Preset/include references are not resolved. Only explicit entries are shown; keys without explicit bindings may still be used by inherited controls.".into());
    }
    if resolve && !includes.is_empty() {
        warnings.push("Include references are recorded but never followed, even when their path names an embedded preset; included entries are not resolved.".into());
    }
    if resolve && !preset_paths.is_empty() {
        warnings.push(format!("Offline preset simulation uses snapshot {}. Controls version 5 is not a game-version match guarantee; older/newer exports may differ. Compatibility aliases, implicit empty presets, game fallbacks (appliedFallbacksVer), default axis values, params and device remapping are not simulated. Unlisted keys are not proven free.", presets::SNAPSHOT));
    }
    if resolve && !resolved_presets.is_empty() {
        warnings.push("Axis properties may have mixed origins: preset_source identifies the entry layer; inherited_properties records retained preset scalars (already included in properties/inputs). Repeated explicit entries remain visible; nested structures use source_path structural matching, not full game-script semantics. Reserved axis groups are retained as structural data, not simulated game axes.".into());
    }
    if bindings.is_empty() {
        warnings.push("No explicit hotkey or axis entries found in the controls block.".into());
    }
    warnings.push("Keyboard codes use DIK physical positions; layout changes legends only. Contexts are structural, not inferred from action names. Shared combinations are not confirmed game conflicts.".into());
    Ok(Profile {
        document,
        bindings,
        preset_paths,
        resolved_presets,
        includes,
        warnings,
    })
}

fn collect_control(control: &Block, budget: &mut usize) -> Result<Vec<Binding>, ImportError> {
    let mut entries = Vec::new();
    for block in control.blocks() {
        let section = match block.name.as_str() {
            "hotkeys" => Section::Hotkeys,
            "axes" => Section::Axes,
            _ => continue,
        };
        collect_bindings(block, section, &[], &mut entries, budget)?;
    }
    Ok(entries)
}

fn charge(budget: &mut usize, bytes: usize) -> Result<(), ImportError> {
    *budget = budget
        .checked_sub(bytes)
        .ok_or(ImportError::StructureTooLarge)?;
    Ok(())
}

// controlspreset.nut at the pinned snapshot: applyBasePreset accepts only
// default; loadHotkeysFromBlk resets once per action per layer, then appends
// every block (including empty siblings). loadAxesFromBlk copies scalar params
// into the existing axis table; an empty axis is therefore not a reset.
fn merge_layer(
    inherited: &mut Vec<Binding>,
    layer: Vec<Binding>,
    budget: &mut usize,
) -> Result<(), ImportError> {
    // Borrow keys rather than duplicating potentially very long structural paths.
    let replaced: BTreeSet<_> = layer.iter().map(|b| b.source_path.as_str()).collect();
    let mut axis_indices = BTreeMap::new();
    for (index, binding) in inherited.iter().enumerate() {
        if binding.section == Section::Axes && replaced.contains(binding.source_path.as_str()) {
            charge(budget, binding.source_path.len())?;
            axis_indices.insert(binding.source_path.clone(), index);
        }
    }
    let old_len = inherited.len();
    // Preserve layer order and repeated explicit entries, not just the last value.
    for binding in &layer {
        charge(budget, binding.source_path.len())?;
    }
    let replaced: BTreeSet<_> = replaced.into_iter().map(str::to_owned).collect();
    for mut binding in layer {
        if binding.section == Section::Axes && !reserved_axis(&binding) {
            if let Some(&index) = axis_indices.get(&binding.source_path) {
                overlay_axis(&inherited[index], &mut binding, budget)?;
            }
            // The next same-layer axis block sees this block's cumulative table.
            if !axis_indices.contains_key(&binding.source_path) {
                charge(budget, binding.source_path.len())?;
            }
            axis_indices.insert(binding.source_path.clone(), inherited.len());
        }
        inherited.push(binding);
    }
    let mut index = 0;
    inherited.retain(|binding| {
        let keep = index >= old_len || !replaced.contains(&binding.source_path);
        index += 1;
        keep
    });
    Ok(())
}

fn reserved_axis(binding: &Binding) -> bool {
    let name = binding
        .context
        .as_deref()
        .unwrap_or(&binding.action)
        .split(" / ")
        .next()
        .unwrap_or("");
    name.starts_with("square") || matches!(name, "mouse" | "devices" | "hangar")
}

fn overlay_axis(
    previous: &Binding,
    current: &mut Binding,
    budget: &mut usize,
) -> Result<(), ImportError> {
    let replaced: BTreeSet<_> = current.properties.iter().map(|p| p.name.as_str()).collect();
    let mut retained = Vec::new();
    for property in &previous.properties {
        if replaced.contains(property.name.as_str()) {
            continue;
        }
        let origin = previous
            .inherited_properties
            .iter()
            .find(|p| p.property == *property)
            .map(|p| &p.preset_source)
            .or(previous.preset_source.as_ref());
        let bytes = property
            .name
            .len()
            .saturating_add(property.kind.len())
            .saturating_add(property.value.len());
        charge(
            budget,
            bytes
                .saturating_mul(3)
                .saturating_add(origin.map_or(0, String::len)),
        )?;
        retained.push(property.clone());
        if let Some(origin) = origin {
            current.inherited_properties.push(InheritedProperty {
                property: property.clone(),
                preset_source: origin.clone(),
            });
        }
    }
    retained.append(&mut current.properties);
    current.properties = retained;
    current.inputs = current
        .properties
        .iter()
        .filter_map(input_from_property)
        .collect();
    current.has_nested_data |= previous.has_nested_data;
    Ok(())
}

fn collect_values(block: &Block, out: &mut Vec<String>) {
    out.extend(block.properties().map(|p| p.value.clone()));
    for child in block.blocks() {
        collect_values(child, out);
    }
}

fn collect_includes(block: &Block, out: &mut Vec<String>) {
    out.extend(
        block
            .properties()
            .filter(|p| p.name == "include")
            .map(|p| p.value.clone()),
    );
    for child in block.blocks() {
        collect_includes(child, out);
    }
}

fn collect_bindings<'a>(
    block: &'a Block,
    section: Section,
    context: &[&'a str],
    out: &mut Vec<Binding>,
    path_budget: &mut usize,
) -> Result<(), ImportError> {
    let context_bytes: usize = context.iter().map(|name| name.len() + 3).sum();
    for child in block.blocks() {
        // A block containing only child blocks is a structural grouping. Direct
        // properties or an empty block constitute an explicit action entry.
        let properties: Vec<_> = child.properties().cloned().collect();
        let has_children = child.blocks().next().is_some();
        if !properties.is_empty() || !has_children {
            // Bound expansion before allocating: a long group name shared by many
            // tiny entries can otherwise amplify a small import into gigabytes.
            let needed = context_bytes
                .saturating_mul(2)
                .saturating_add(child.name.len())
                .saturating_add(64);
            *path_budget = path_budget
                .checked_sub(needed)
                .ok_or(ImportError::StructureTooLarge)?;
            let inputs = properties.iter().filter_map(input_from_property).collect();
            let mut path = vec!["controls", block_section(section)];
            path.extend_from_slice(context);
            path.push(child.name.as_str());
            out.push(Binding {
                action: child.name.clone(),
                context: (!context.is_empty()).then(|| context.join(" / ")),
                section,
                inputs,
                properties,
                has_nested_data: has_children,
                source_path: path.join(" / "),
                preset_source: None,
                inherited_properties: Vec::new(),
            });
        }
        if has_children {
            let mut nested = context.to_vec();
            nested.push(child.name.as_str());
            collect_bindings(child, section, &nested, out, path_budget)?;
        }
    }
    Ok(())
}

fn block_section(section: Section) -> &'static str {
    match section {
        Section::Hotkeys => "hotkeys",
        Section::Axes => "axes",
    }
}

fn input_from_property(p: &Property) -> Option<Input> {
    // A familiar name with an unfamiliar type is not silently coerced.
    let code = (p.kind == "i" || p.kind == "i64")
        .then(|| p.value.parse::<i32>().ok())
        .flatten();
    let known = matches!(
        p.name.as_str(),
        "keyboardKey"
            | "mouseButton"
            | "joyButton"
            | "mouseAxisId"
            | "joyAxis"
            | "joyAxisId"
            | "axisId"
    );
    let possible = known || {
        let name = p.name.to_ascii_lowercase();
        name.contains("key")
            || name.contains("button")
            || name.contains("axis") && !name.contains("deadzone")
    };
    if !possible {
        return None;
    }
    Some(match (p.name.as_str(), code) {
        ("keyboardKey", Some(code)) => Input::Keyboard(code),
        ("mouseButton", Some(code)) => Input::MouseButton(code),
        ("joyButton", Some(code)) => Input::GamepadButton(code),
        ("mouseAxisId" | "joyAxis" | "joyAxisId" | "axisId", Some(code)) => Input::Axis {
            kind: p.name.clone(),
            code,
        },
        _ => Input::Unknown {
            kind: format!("{}:{}", p.name, p.kind),
            value: p.value.clone(),
        },
    })
}

#[derive(Debug)]
pub struct Analysis {
    pub by_key: BTreeMap<i32, Vec<usize>>,
    pub issues: Vec<Vec<BindingIssue>>,
    /// Explicitly surfaced by the GUI; key usage remains complete.
    pub overlap_checks_truncated: bool,
}

impl Analysis {
    pub fn indices_for_key(&self, code: i32) -> &[usize] {
        self.by_key
            .get(&code)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }
}

/// Compare full combinations, not individual shared modifier/main keys.
/// Unknown input encodings and disabled axes do not support overlap conclusions.
pub fn analyze(profile: &Profile) -> Analysis {
    let mut by_key: BTreeMap<i32, Vec<usize>> = BTreeMap::new();
    let mut combinations: BTreeMap<Vec<Input>, Vec<usize>> = BTreeMap::new();
    for (index, binding) in profile.bindings.iter().enumerate() {
        let keys: BTreeSet<_> = binding
            .inputs
            .iter()
            .filter_map(|input| match input {
                Input::Keyboard(code) => Some(*code),
                _ => None,
            })
            .collect();
        for code in keys {
            by_key.entry(code).or_default().push(index);
        }
        if binding.inputs.is_empty()
            || binding.inputs.iter().any(|i| {
                matches!(
                    i,
                    Input::Unknown { .. }
                        | Input::Axis {
                            code: i32::MIN..=-1,
                            ..
                        }
                )
            })
        {
            continue;
        }
        let mut combination = binding.inputs.clone();
        combination.sort();
        combinations.entry(combination).or_default().push(index);
    }
    let mut issues = vec![Vec::new(); profile.bindings.len()];
    // Avoid quadratic time/memory for malicious exports with one repeated combo.
    let mut remaining_checks = 100_000usize;
    let mut overlap_checks_truncated = false;
    'groups: for indices in combinations.values() {
        for (offset, &left) in indices.iter().enumerate() {
            for &right in &indices[offset + 1..] {
                if remaining_checks == 0 {
                    overlap_checks_truncated = true;
                    break 'groups;
                }
                remaining_checks -= 1;
                let a = &profile.bindings[left];
                let b = &profile.bindings[right];
                if a.section != b.section {
                    continue;
                }
                let kind = if a.action == b.action
                    && a.context == b.context
                    && !a.has_nested_data
                    && !b.has_nested_data
                    && a.inherited_properties.is_empty()
                    && b.inherited_properties.is_empty()
                    && a.properties == b.properties
                {
                    IssueKind::Duplicate
                } else if let (Some(a), Some(b)) = (&a.context, &b.context) {
                    if a != b {
                        continue;
                    }
                    IssueKind::PotentialConflict
                } else {
                    IssueKind::ContextUnknown
                };
                issues[left].push(BindingIssue { kind, other: right });
                issues[right].push(BindingIssue { kind, other: left });
            }
        }
    }
    Analysis {
        by_key,
        issues,
        overlap_checks_truncated,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(hotkeys: &str) -> Profile {
        import(&format!("controls {{ hotkeys {{ {hotkeys} }} }}")).unwrap()
    }

    #[test]
    fn combinations_are_not_alternatives() {
        let p = profile("A {keyboardKey:i=29;keyboardKey:i=48}\nA {keyboardKey:i=57}");
        assert_eq!(p.bindings.len(), 2);
        assert_eq!(p.bindings[0].inputs.len(), 2);
        let a = analyze(&p);
        assert_eq!(a.indices_for_key(29), &[0]);
        assert_eq!(a.indices_for_key(57), &[1]);
        assert!(a.issues.iter().all(Vec::is_empty));
    }

    #[test]
    fn duplicate_and_unknown_context_overlap_are_distinct() {
        let p = profile("A {keyboardKey:i=57}\nA {keyboardKey:i=57}\nB {keyboardKey:i=57}");
        let a = analyze(&p);
        assert_eq!(a.issues[0][0].kind, IssueKind::Duplicate);
        assert_eq!(a.issues[0][1].kind, IssueKind::ContextUnknown);
    }

    #[test]
    fn reused_modifier_is_not_a_conflict() {
        let p =
            profile("A {keyboardKey:i=29;keyboardKey:i=48}\nB {keyboardKey:i=29;keyboardKey:i=49}");
        let a = analyze(&p);
        assert_eq!(a.indices_for_key(29), &[0, 1]);
        assert!(a.issues.iter().all(Vec::is_empty));
    }

    #[test]
    fn known_different_contexts_do_not_conflict() {
        let p = profile(
            "air {A {keyboardKey:i=57}}\ntank {B {keyboardKey:i=57}}\nair {C {keyboardKey:i=57}}",
        );
        let a = analyze(&p);
        assert_eq!(a.issues[0].len(), 1);
        assert_eq!(a.issues[0][0].kind, IssueKind::PotentialConflict);
        assert!(a.issues[1].is_empty());
    }

    #[test]
    fn unknowns_empty_overrides_and_metadata_survive() {
        let p = profile(
            "A {}\nA {keyboardKey:i=999;newButton:x=opaque;activationType:i=2}\nB {keyboardKey:t=abc}",
        );
        assert!(p.bindings[0].inputs.is_empty());
        assert_eq!(p.bindings[1].properties.len(), 3);
        assert_eq!(analyze(&p).indices_for_key(999), &[1]);
        assert!(matches!(p.bindings[2].inputs[0], Input::Unknown { .. }));
    }

    #[test]
    fn metadata_prevents_false_exact_duplicate() {
        let p = profile(
            "A {keyboardKey:i=42;activationType:i=1}\nA {keyboardKey:i=42;activationType:i=2}",
        );
        assert_eq!(analyze(&p).issues[0][0].kind, IssueKind::ContextUnknown);
    }

    #[test]
    fn includes_are_recorded_without_following_paths() {
        let p = import("include \"missing.blk\"\ncontrols {basePresetPaths {default:t=\"base.blk\"} hotkeys {A{}}}").unwrap();
        assert_eq!(p.preset_paths, ["base.blk"]);
        assert_eq!(p.includes, ["missing.blk"]);
        assert!(!p.warnings.is_empty());
    }

    #[test]
    fn axis_settings_and_disabled_axes_do_not_overlap() {
        let p = import("controls {axes {zoom {mouseAxisId:i=-1} pitch {mouseAxisId:i=-1} roll {nonlinearity:r=1}}}").unwrap();
        assert_eq!(p.bindings.len(), 3);
        assert!(p.bindings[2].inputs.is_empty());
        assert!(analyze(&p).issues.iter().all(Vec::is_empty));
    }

    #[test]
    fn nested_data_differences_are_not_asserted_to_be_duplicates() {
        let p = profile(
            "A {keyboardKey:i=57; extra {mode:i=1}}\nA {keyboardKey:i=57; extra {mode:i=2}}",
        );
        assert!(p.bindings[0].has_nested_data);
        assert!(
            analyze(&p)
                .issues
                .iter()
                .flatten()
                .all(|issue| issue.kind != IssueKind::Duplicate)
        );
    }

    #[test]
    fn derived_path_budget_prevents_small_input_amplification() {
        let p = blk::parse("hotkeys {group {A{} B{}}}").unwrap();
        let mut budget = 80;
        let mut bindings = Vec::new();
        assert!(matches!(
            collect_bindings(
                p.blocks().next().unwrap(),
                Section::Hotkeys,
                &[],
                &mut bindings,
                &mut budget
            ),
            Err(ImportError::StructureTooLarge)
        ));
    }

    #[test]
    fn merging_matches_full_structural_paths_and_retains_sibling_order() {
        let mut inherited =
            profile("air {A{keyboardKey:i=7} A{keyboardKey:i=8}} tank {A{keyboardKey:i=9}}")
                .bindings;
        let layer = profile("air {A{} A{keyboardKey:i=10} A{}}").bindings;
        merge_layer(&mut inherited, layer, &mut (64 * 1024 * 1024)).unwrap();
        assert_eq!(inherited.len(), 4);
        assert_eq!(inherited[0].context.as_deref(), Some("tank"));
        assert_eq!(inherited[0].inputs, [Input::Keyboard(9)]);
        assert!(inherited[1].inputs.is_empty());
        assert_eq!(inherited[2].inputs, [Input::Keyboard(10)]);
        assert!(inherited[3].inputs.is_empty());
        assert!(
            inherited[1..]
                .iter()
                .all(|b| b.source_path == "controls / hotkeys / air / A")
        );
    }

    #[test]
    fn inheritance_path_and_property_copies_share_the_derived_budget() {
        let mut inherited =
            import_explicit("controls {axes {zoom{mouseAxisId:i=3;relative:b=yes}}}")
                .unwrap()
                .bindings;
        inherited[0].preset_source = Some("config/hotkeys/test-fixture.blk".into());
        let layer = import_explicit("controls {axes {zoom{}}}")
            .unwrap()
            .bindings;
        assert!(matches!(
            merge_layer(&mut inherited, layer, &mut 1),
            Err(ImportError::StructureTooLarge)
        ));

        let previous = &inherited[0];
        let mut current = previous.clone();
        current.properties.clear();
        assert!(matches!(
            overlay_axis(previous, &mut current, &mut 1),
            Err(ImportError::StructureTooLarge)
        ));
    }

    #[test]
    fn pathological_overlap_work_is_bounded_and_disclosed() {
        let p = profile(&"A{keyboardKey:i=57}\n".repeat(1000));
        let a = analyze(&p);
        assert!(a.overlap_checks_truncated);
        assert_eq!(a.indices_for_key(57).len(), 1000);
        assert_eq!(a.issues.iter().map(Vec::len).sum::<usize>(), 200_000);
    }

    #[test]
    fn rejects_non_controls_documents() {
        assert!(matches!(
            import("settings{}"),
            Err(ImportError::MissingControls)
        ));
    }
}
