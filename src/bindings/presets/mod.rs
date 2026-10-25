//! Exact offline allowlist; paths are identifiers, never filesystem paths.
use std::sync::OnceLock;

use crate::blk::{self, Block};

pub(crate) const SNAPSHOT: &str = "aa326f10c6ba7d523647572368ead2e4d51ffb4d (2.59.0.86)";
const MAX_DEPTH: usize = 16;
const MAX_VISITS: usize = 64;

fn registry() -> &'static [(String, Block)] {
    static DOCUMENTS: OnceLock<Vec<(String, Block)>> = OnceLock::new();
    DOCUMENTS.get_or_init(|| {
        [
            (
                "config/hotkeys/hotkey.keyboard_ver1.blk",
                include_str!("data/hotkey.keyboard_ver1.blk"),
            ),
            (
                "config/hotkeys/hotkey.base.keyboard_simulator.blk",
                include_str!("data/hotkey.base.keyboard_simulator.blk"),
            ),
            (
                "config/hotkeys/hotkey.base.keyboard.blk",
                include_str!("data/hotkey.base.keyboard.blk"),
            ),
            (
                "config/hotkeys/hotkey.empty_ver1.blk",
                include_str!("data/hotkey.empty_ver1.blk"),
            ),
            (
                "config/hotkeys/hotkey.event.human.blk",
                include_str!("data/hotkey.event.human.blk"),
            ),
        ]
        .into_iter()
        .map(|(path, text)| {
            (
                path.to_owned(),
                blk::parse(text).expect("validated embedded preset"),
            )
        })
        .collect()
    })
}

pub(crate) fn version_five(controls: &Block) -> bool {
    controls
        .property("version")
        .is_some_and(|p| matches!(p.kind.as_str(), "i" | "i64") && p.value.parse::<i64>() == Ok(5))
}

pub(crate) struct Layer<'a> {
    pub controls: &'a Block,
    pub path: String,
}

pub(crate) fn resolve<'a>(controls: &Block, warnings: &mut Vec<String>) -> Vec<Layer<'a>> {
    resolve_with(
        controls,
        &|path| {
            registry()
                .iter()
                .find(|(name, _)| name == path)
                .map(|(_, doc)| doc)
        },
        warnings,
    )
}

// Kept injectable so cycles and exhaustion can be tested without expanding the
// production allowlist or adding runtime I/O.
fn resolve_with<'a>(
    controls: &Block,
    lookup: &impl Fn(&str) -> Option<&'a Block>,
    warnings: &mut Vec<String>,
) -> Vec<Layer<'a>> {
    let mut state = Resolver {
        lookup,
        warnings,
        stack: Vec::new(),
        remaining: MAX_VISITS,
        layers: Vec::new(),
    };
    if !state.bases(controls) {
        state.layers.clear();
        state.warnings.push("Incomplete preset inheritance: the affected base chain was not applied; explicit export entries are retained.".into());
    }
    state.layers
}

struct Resolver<'a, 'b, F> {
    lookup: &'b F,
    warnings: &'b mut Vec<String>,
    stack: Vec<String>,
    remaining: usize,
    layers: Vec<Layer<'a>>,
}

impl<'a, F: Fn(&str) -> Option<&'a Block>> Resolver<'a, '_, F> {
    fn bases(&mut self, controls: &Block) -> bool {
        let mut complete = true;
        for bases in controls.blocks().filter(|b| b.name == "basePresetPaths") {
            for property in bases.properties() {
                if property.name != "default" || property.kind != "t" {
                    self.warnings.push(format!(
                        "Ignored non-default or non-text preset reference {}:{}={:?}; only default:t is merged.",
                        property.name, property.kind, property.value
                    ));
                    continue;
                }
                if !version_five(controls) {
                    self.warnings.push("Preset inheritance not resolved: only controls version 5 is supported; explicit entries retained (no guessed legacy inheritance).".into());
                    complete = false;
                    continue;
                }
                complete &= self.visit(&property.value);
            }
            if bases.blocks().next().is_some() {
                self.warnings.push("Ignored nested basePresetPaths references; only direct default:t properties are merged.".into());
            }
        }
        complete
    }

    fn visit(&mut self, path: &str) -> bool {
        if self.remaining == 0 || self.stack.len() >= MAX_DEPTH {
            self.warnings.push(format!(
                "Preset depth/visit budget exceeded at {path:?}; reference not resolved."
            ));
            return false;
        }
        self.remaining -= 1;
        if self.stack.iter().any(|ancestor| ancestor == path) {
            self.warnings.push(format!(
                "Preset cycle detected at {path:?}; reference not resolved."
            ));
            return false;
        }
        let Some(document) = (self.lookup)(path) else {
            self.warnings.push(format!("Unknown preset {path:?}: not in the exact embedded allowlist; reference not resolved (no filesystem or network lookup)."));
            return false;
        };
        let controls: Vec<_> = document.blocks().filter(|b| b.name == "controls").collect();
        if controls.len() != 1 || !version_five(controls[0]) {
            self.warnings.push(format!("Embedded preset {path:?} is not a single version 5 controls document; reference not resolved."));
            return false;
        }
        self.stack.push(path.to_owned());
        let complete = self.bases(controls[0]);
        self.stack.pop();
        if complete {
            self.layers.push(Layer {
                controls: controls[0],
                path: path.to_owned(),
            });
        }
        complete
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycles_unknown_dependencies_and_limits_discard_partial_chains() {
        let root = blk::parse("controls {version:i=5; basePresetPaths {default:t=a}} ").unwrap();
        let a = blk::parse("controls {version:i=5; basePresetPaths {default:t=b}} ").unwrap();
        let b = blk::parse("controls {version:i=5; basePresetPaths {default:t=a}} ").unwrap();
        let controls = root.blocks().next().unwrap();
        let mut warnings = Vec::new();
        assert!(
            resolve_with(
                controls,
                &|p| match p {
                    "a" => Some(&a),
                    "b" => Some(&b),
                    _ => None,
                },
                &mut warnings
            )
            .is_empty()
        );
        assert!(warnings.iter().any(|w| w.contains("cycle")));
        warnings.clear();
        assert!(resolve_with(controls, &|p| (p == "a").then_some(&a), &mut warnings).is_empty());
        assert!(warnings.iter().any(|w| w.contains("Unknown preset")));

        let chain: Vec<_> = (0..MAX_DEPTH + 2)
            .map(|n| {
                blk::parse(&format!(
                    "controls {{version:i=5; basePresetPaths {{default:t=\"{}\"}}}}",
                    n + 1
                ))
                .unwrap()
            })
            .collect();
        warnings.clear();
        assert!(
            resolve_with(
                chain[0].blocks().next().unwrap(),
                &|p| p.parse::<usize>().ok().and_then(|n| chain.get(n)),
                &mut warnings
            )
            .is_empty()
        );
        assert!(warnings.iter().any(|w| w.contains("budget")));

        let terminal = blk::parse("controls {version:i=5}").unwrap();
        let many = blk::parse(&format!(
            "controls {{version:i=5; basePresetPaths {{{}}}}}",
            "default:t=a;".repeat(MAX_VISITS + 1)
        ))
        .unwrap();
        warnings.clear();
        assert!(
            resolve_with(
                many.blocks().next().unwrap(),
                &|_| Some(&terminal),
                &mut warnings
            )
            .is_empty()
        );
        assert!(warnings.iter().any(|w| w.contains("budget")));
    }
}
