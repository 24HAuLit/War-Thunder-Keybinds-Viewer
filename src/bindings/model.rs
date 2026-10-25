use crate::blk::{Block, Property};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Section {
    Hotkeys,
    Axes,
}

impl Section {
    pub fn name(self) -> &'static str {
        match self {
            Self::Hotkeys => "Hotkeys",
            Self::Axes => "Axes",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Input {
    Keyboard(i32),
    MouseButton(i32),
    GamepadButton(i32),
    Axis { kind: String, code: i32 },
    Unknown { kind: String, value: String },
}

impl Input {
    pub fn device(&self) -> &'static str {
        match self {
            Self::Keyboard(_) => "Keyboard",
            Self::MouseButton(_) => "Mouse",
            Self::GamepadButton(_) => "Gamepad",
            Self::Axis { kind, .. } if kind.starts_with("mouse") => "Mouse",
            Self::Axis { .. } => "Gamepad / axis",
            Self::Unknown { .. } => "Unknown",
        }
    }

    pub fn label(&self, layout: super::keymap::Layout) -> String {
        match self {
            Self::Keyboard(code) => super::keymap::key_label(*code, layout),
            Self::MouseButton(code) => format!("Mouse button {code}"),
            Self::GamepadButton(code) => format!("Joystick button {code} (export index)"),
            Self::Axis { kind, code } if *code < 0 => format!("{kind}: {code} (disabled / unset)"),
            Self::Axis { kind, code } => format!("{kind}: {code}"),
            Self::Unknown { kind, value } => format!("{kind}={value} (unknown)"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Binding {
    pub action: String,
    /// Structural context only; never guessed from the action's name.
    pub context: Option<String>,
    pub section: Section,
    pub inputs: Vec<Input>,
    /// All direct properties, including activation modes and unfamiliar fields.
    pub properties: Vec<Property>,
    /// Nested blocks remain in the document; do not assert leaf-entry identity.
    pub has_nested_data: bool,
    pub source_path: String,
    /// Layer supplying this entry: None for the export, otherwise an exact
    /// config/hotkeys/... allowlisted path. See inherited_properties for axes.
    pub preset_source: Option<String>,
    /// Effective scalar properties retained from earlier preset layers. These
    /// are also present in properties; provenance is metadata, not extra inputs.
    pub inherited_properties: Vec<InheritedProperty>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InheritedProperty {
    pub property: Property,
    pub preset_source: String,
}

impl Binding {
    pub fn combination(&self, layout: super::keymap::Layout) -> String {
        if self.inputs.is_empty() {
            return "No explicit input (empty override / settings only)".into();
        }
        self.inputs
            .iter()
            .map(|input| input.label(layout))
            .collect::<Vec<_>>()
            .join(" + ")
    }

    /// Exact English game label from the bundled catalogue, when available.
    pub fn english_name(&self) -> Option<&'static str> {
        super::localization::english_name(&self.action, self.section)
    }

    /// English game label, or an identifier-based fallback for unknown commands.
    pub fn display_name(&self) -> String {
        self.english_name().map(str::to_owned).unwrap_or_else(|| {
            self.action
                .strip_prefix("ID_")
                .unwrap_or(&self.action)
                .replace('_', " ")
                .to_lowercase()
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueKind {
    Duplicate,
    PotentialConflict,
    ContextUnknown,
}

impl IssueKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Duplicate => "Exact duplicate entry",
            Self::PotentialConflict => "Potential overlap in the same context",
            Self::ContextUnknown => "Shared combination; context unknown",
        }
    }
}

#[derive(Debug, Clone)]
pub struct BindingIssue {
    pub kind: IssueKind,
    pub other: usize,
}

#[derive(Debug)]
pub struct Profile {
    /// Lossless in entry order and values, but not whitespace/comments.
    pub document: Block,
    pub bindings: Vec<Binding>,
    pub preset_paths: Vec<String>,
    /// Successfully applied embedded paths, unique and in base-first order.
    pub resolved_presets: Vec<String>,
    pub includes: Vec<String>,
    pub warnings: Vec<String>,
}
