//! DirectInput physical keys with explicitly selected display legends.
//! Layout selection never translates a binding's scan code or infers an OS layout.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Layout {
    #[default]
    Qwerty,
    Azerty,
}

impl Layout {
    pub fn name(self) -> &'static str {
        match self {
            Self::Qwerty => "QWERTY",
            Self::Azerty => "AZERTY",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Key {
    pub code: i32,
    /// Width in units of an ordinary letter key.
    pub width: f32,
}

// Gaijin's Dagor Engine DKEY scancodes match these DirectInput DIK offsets:
// https://github.com/GaijinEntertainment/DagorEngine/blob/main/prog/dagorInclude/drv/hid/dag_hiKeybIds.h
// Also verified against Microsoft's Windows SDK dinput.h:
// https://github.com/microsoft/win32metadata/blob/main/generation/WinSDK/RecompiledIdlHeaders/um/dinput.h
// These are DIK offsets, NOT virtual keys, USB usages, or raw E0-prefixed codes.
// Legends are unshifted US QWERTY / traditional French (France) AZERTY;
// letters are uppercase for readability. The extra ISO key uses \ / <.
// Regional, media and system keys remain recognizable even when not drawn.
const KEY_LABELS: &[(i32, &str, &str)] = &[
    (0x01, "Escape", "Escape"),
    (0x02, "1", "&"),
    (0x03, "2", "é"),
    (0x04, "3", "\""),
    (0x05, "4", "'"),
    (0x06, "5", "("),
    (0x07, "6", "-"),
    (0x08, "7", "è"),
    (0x09, "8", "_"),
    (0x0a, "9", "ç"),
    (0x0b, "0", "à"),
    (0x0c, "-", ")"),
    (0x0d, "=", "="),
    (0x0e, "Backspace", "Backspace"),
    (0x0f, "Tab", "Tab"),
    (0x10, "Q", "A"),
    (0x11, "W", "Z"),
    (0x12, "E", "E"),
    (0x13, "R", "R"),
    (0x14, "T", "T"),
    (0x15, "Y", "Y"),
    (0x16, "U", "U"),
    (0x17, "I", "I"),
    (0x18, "O", "O"),
    (0x19, "P", "P"),
    (0x1a, "[", "^"),
    (0x1b, "]", "$"),
    (0x1c, "Enter", "Enter"),
    (0x1d, "Left Ctrl", "Left Ctrl"),
    (0x1e, "A", "Q"),
    (0x1f, "S", "S"),
    (0x20, "D", "D"),
    (0x21, "F", "F"),
    (0x22, "G", "G"),
    (0x23, "H", "H"),
    (0x24, "J", "J"),
    (0x25, "K", "K"),
    (0x26, "L", "L"),
    (0x27, ";", "M"),
    (0x28, "'", "ù"),
    (0x29, "`", "²"),
    (0x2a, "Left Shift", "Left Shift"),
    (0x2b, "\\", "*"),
    (0x2c, "Z", "W"),
    (0x2d, "X", "X"),
    (0x2e, "C", "C"),
    (0x2f, "V", "V"),
    (0x30, "B", "B"),
    (0x31, "N", "N"),
    (0x32, "M", ","),
    (0x33, ",", ";"),
    (0x34, ".", ":"),
    (0x35, "/", "!"),
    (0x36, "Right Shift", "Right Shift"),
    (0x37, "Numpad *", "Numpad *"),
    (0x38, "Left Alt", "Left Alt"),
    (0x39, "Space", "Space"),
    (0x3a, "Caps Lock", "Caps Lock"),
    (0x3b, "F1", "F1"),
    (0x3c, "F2", "F2"),
    (0x3d, "F3", "F3"),
    (0x3e, "F4", "F4"),
    (0x3f, "F5", "F5"),
    (0x40, "F6", "F6"),
    (0x41, "F7", "F7"),
    (0x42, "F8", "F8"),
    (0x43, "F9", "F9"),
    (0x44, "F10", "F10"),
    (0x45, "Num Lock", "Num Lock"),
    (0x46, "Scroll Lock", "Scroll Lock"),
    (0x47, "Numpad 7", "Numpad 7"),
    (0x48, "Numpad 8", "Numpad 8"),
    (0x49, "Numpad 9", "Numpad 9"),
    (0x4a, "Numpad -", "Numpad -"),
    (0x4b, "Numpad 4", "Numpad 4"),
    (0x4c, "Numpad 5", "Numpad 5"),
    (0x4d, "Numpad 6", "Numpad 6"),
    (0x4e, "Numpad +", "Numpad +"),
    (0x4f, "Numpad 1", "Numpad 1"),
    (0x50, "Numpad 2", "Numpad 2"),
    (0x51, "Numpad 3", "Numpad 3"),
    (0x52, "Numpad 0", "Numpad 0"),
    (0x53, "Numpad .", "Numpad ."),
    (0x56, "\\", "<"),
    (0x57, "F11", "F11"),
    (0x58, "F12", "F12"),
    (0x64, "F13", "F13"),
    (0x65, "F14", "F14"),
    (0x66, "F15", "F15"),
    (0x70, "Kana", "Kana"),
    (0x73, "ABNT C1", "ABNT C1"),
    (0x79, "Convert", "Convert"),
    (0x7b, "No Convert", "No Convert"),
    (0x7d, "Yen", "Yen"),
    (0x7e, "ABNT C2", "ABNT C2"),
    (0x8d, "Numpad =", "Numpad ="),
    (0x90, "Previous Track", "Previous Track"),
    (0x91, "@ (PC-98)", "@ (PC-98)"),
    (0x92, ": (PC-98)", ": (PC-98)"),
    (0x93, "_ (PC-98)", "_ (PC-98)"),
    (0x94, "Kanji", "Kanji"),
    (0x95, "Stop (PC-98)", "Stop (PC-98)"),
    (0x96, "AX", "AX"),
    (0x97, "Unlabeled", "Unlabeled"),
    (0x99, "Next Track", "Next Track"),
    (0x9c, "Numpad Enter", "Numpad Enter"),
    (0x9d, "Right Ctrl", "Right Ctrl"),
    (0xa0, "Mute", "Mute"),
    (0xa1, "Calculator", "Calculator"),
    (0xa2, "Play / Pause", "Play / Pause"),
    (0xa4, "Media Stop", "Media Stop"),
    (0xae, "Volume Down", "Volume Down"),
    (0xb0, "Volume Up", "Volume Up"),
    (0xb2, "Web Home", "Web Home"),
    (0xb3, "Numpad ,", "Numpad ,"),
    (0xb5, "Numpad /", "Numpad /"),
    (0xb7, "Print Screen", "Print Screen"),
    (0xb8, "Right Alt", "Right Alt"),
    (0xc5, "Pause", "Pause"),
    (0xc7, "Home", "Home"),
    (0xc8, "Up", "Up"),
    (0xc9, "Page Up", "Page Up"),
    (0xcb, "Left", "Left"),
    (0xcd, "Right", "Right"),
    (0xcf, "End", "End"),
    (0xd0, "Down", "Down"),
    (0xd1, "Page Down", "Page Down"),
    (0xd2, "Insert", "Insert"),
    (0xd3, "Delete", "Delete"),
    (0xdb, "Left Win", "Left Win"),
    (0xdc, "Right Win", "Right Win"),
    (0xdd, "Menu", "Menu"),
    (0xde, "Power", "Power"),
    (0xdf, "Sleep", "Sleep"),
    (0xe3, "Wake", "Wake"),
    (0xe5, "Web Search", "Web Search"),
    (0xe6, "Web Favorites", "Web Favorites"),
    (0xe7, "Web Refresh", "Web Refresh"),
    (0xe8, "Web Stop", "Web Stop"),
    (0xe9, "Web Forward", "Web Forward"),
    (0xea, "Web Back", "Web Back"),
    (0xeb, "My Computer", "My Computer"),
    (0xec, "Mail", "Mail"),
    (0xed, "Media Select", "Media Select"),
];

/// Returns a legend without changing or normalizing the physical DIK code.
/// Unknown values retain the entire signed input, including out-of-range values.
pub fn key_label(code: i32, layout: Layout) -> String {
    match KEY_LABELS.iter().find(|&&(known, _, _)| known == code) {
        Some(&(_, qwerty, azerty)) => match layout {
            Layout::Qwerty => qwerty,
            Layout::Azerty => azerty,
        }
        .to_owned(),
        None => format!("Unknown key ({code})"),
    }
}

pub fn is_known(code: i32) -> bool {
    KEY_LABELS.iter().any(|&(known, _, _)| known == code)
}

/// A full-size 105-key ISO-style keyboard, independent of the chosen legends.
/// The first six rows are the function/main block; navigation and arrows follow,
/// then five numpad rows. Enter and numpad tall keys are flattened because `Key`
/// has width but no height; no placeholder or duplicate scan codes are emitted.
pub fn keyboard_rows() -> Vec<Vec<Key>> {
    const ROWS: &[&[(i32, f32)]] = &[
        &[
            (1, 1.0),
            (59, 1.0),
            (60, 1.0),
            (61, 1.0),
            (62, 1.0),
            (63, 1.0),
            (64, 1.0),
            (65, 1.0),
            (66, 1.0),
            (67, 1.0),
            (68, 1.0),
            (87, 1.0),
            (88, 1.0),
            (183, 1.0),
            (70, 1.0),
            (197, 1.0),
        ],
        &[
            (41, 1.0),
            (2, 1.0),
            (3, 1.0),
            (4, 1.0),
            (5, 1.0),
            (6, 1.0),
            (7, 1.0),
            (8, 1.0),
            (9, 1.0),
            (10, 1.0),
            (11, 1.0),
            (12, 1.0),
            (13, 1.0),
            (14, 2.0),
        ],
        &[
            (15, 1.5),
            (16, 1.0),
            (17, 1.0),
            (18, 1.0),
            (19, 1.0),
            (20, 1.0),
            (21, 1.0),
            (22, 1.0),
            (23, 1.0),
            (24, 1.0),
            (25, 1.0),
            (26, 1.0),
            (27, 1.0),
            (43, 1.5),
        ],
        &[
            (58, 1.75),
            (30, 1.0),
            (31, 1.0),
            (32, 1.0),
            (33, 1.0),
            (34, 1.0),
            (35, 1.0),
            (36, 1.0),
            (37, 1.0),
            (38, 1.0),
            (39, 1.0),
            (40, 1.0),
            (28, 2.25),
        ],
        &[
            (42, 1.25),
            (86, 1.0),
            (44, 1.0),
            (45, 1.0),
            (46, 1.0),
            (47, 1.0),
            (48, 1.0),
            (49, 1.0),
            (50, 1.0),
            (51, 1.0),
            (52, 1.0),
            (53, 1.0),
            (54, 2.75),
        ],
        &[
            (29, 1.25),
            (219, 1.25),
            (56, 1.25),
            (57, 6.25),
            (184, 1.25),
            (220, 1.25),
            (221, 1.25),
            (157, 1.25),
        ],
        &[(210, 1.0), (199, 1.0), (201, 1.0)],
        &[(211, 1.0), (207, 1.0), (209, 1.0)],
        &[(200, 1.0)],
        &[(203, 1.0), (208, 1.0), (205, 1.0)],
        &[(69, 1.0), (181, 1.0), (55, 1.0), (74, 1.0)],
        &[(71, 1.0), (72, 1.0), (73, 1.0), (78, 1.0)],
        &[(75, 1.0), (76, 1.0), (77, 1.0)],
        &[(79, 1.0), (80, 1.0), (81, 1.0), (156, 1.0)],
        &[(82, 2.0), (83, 1.0)],
    ];

    ROWS.iter()
        .map(|row| {
            row.iter()
                .map(|&(code, width)| Key { code, width })
                .collect()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn layout_names_and_default_are_explicit() {
        assert_eq!(Layout::default(), Layout::Qwerty);
        assert_eq!(Layout::Qwerty.name(), "QWERTY");
        assert_eq!(Layout::Azerty.name(), "AZERTY");
    }

    #[test]
    fn directinput_controls_and_extended_keys() {
        let expected = [
            (1, "Escape"),
            (28, "Enter"),
            (57, "Space"),
            (29, "Left Ctrl"),
            (42, "Left Shift"),
            (54, "Right Shift"),
            (56, "Left Alt"),
            (157, "Right Ctrl"),
            (184, "Right Alt"),
            (183, "Print Screen"),
            (197, "Pause"),
            (199, "Home"),
            (200, "Up"),
            (201, "Page Up"),
            (203, "Left"),
            (205, "Right"),
            (207, "End"),
            (208, "Down"),
            (209, "Page Down"),
            (210, "Insert"),
            (211, "Delete"),
            (219, "Left Win"),
            (220, "Right Win"),
            (221, "Menu"),
        ];
        for (code, label) in expected {
            assert!(is_known(code));
            for layout in [Layout::Qwerty, Layout::Azerty] {
                assert_eq!(key_label(code, layout), label, "DIK {code}");
            }
        }
        for (code, number) in (59..=68).zip(1..=10) {
            assert_eq!(key_label(code, Layout::Qwerty), format!("F{number}"));
        }
        assert_eq!(key_label(87, Layout::Qwerty), "F11");
        assert_eq!(key_label(88, Layout::Qwerty), "F12");
    }

    #[test]
    fn french_azerty_letters_and_punctuation_are_position_based() {
        let expected = [
            (16, "Q", "A"),
            (30, "A", "Q"),
            (17, "W", "Z"),
            (44, "Z", "W"),
            (39, ";", "M"),
            (50, "M", ","),
            (51, ",", ";"),
            (52, ".", ":"),
            (53, "/", "!"),
            (26, "[", "^"),
            (27, "]", "$"),
            (40, "'", "ù"),
            (41, "`", "²"),
            (43, "\\", "*"),
            (86, "\\", "<"),
        ];
        for (code, qwerty, azerty) in expected {
            assert_eq!(key_label(code, Layout::Qwerty), qwerty);
            assert_eq!(key_label(code, Layout::Azerty), azerty);
        }
    }

    #[test]
    fn french_number_row_uses_unshifted_legends() {
        let qwerty = ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "-", "="];
        let azerty = ["&", "é", "\"", "'", "(", "-", "è", "_", "ç", "à", ")", "="];
        for (code, (qwerty, azerty)) in (2..=13).zip(qwerty.into_iter().zip(azerty)) {
            assert_eq!(key_label(code, Layout::Qwerty), qwerty);
            assert_eq!(key_label(code, Layout::Azerty), azerty);
        }
    }

    #[test]
    fn numpad_is_distinct_from_main_keys_and_navigation() {
        let expected = [
            (69, "Num Lock"),
            (71, "Numpad 7"),
            (72, "Numpad 8"),
            (73, "Numpad 9"),
            (75, "Numpad 4"),
            (76, "Numpad 5"),
            (77, "Numpad 6"),
            (79, "Numpad 1"),
            (80, "Numpad 2"),
            (81, "Numpad 3"),
            (82, "Numpad 0"),
            (83, "Numpad ."),
            (55, "Numpad *"),
            (74, "Numpad -"),
            (78, "Numpad +"),
            (156, "Numpad Enter"),
            (181, "Numpad /"),
            (141, "Numpad ="),
            (179, "Numpad ,"),
        ];
        for (code, label) in expected {
            for layout in [Layout::Qwerty, Layout::Azerty] {
                assert_eq!(key_label(code, layout), label);
            }
        }
    }

    #[test]
    fn international_and_media_codes_are_known_without_inference() {
        for (code, label) in [
            (112, "Kana"),
            (115, "ABNT C1"),
            (121, "Convert"),
            (123, "No Convert"),
            (125, "Yen"),
            (126, "ABNT C2"),
            (144, "Previous Track"),
            (148, "Kanji"),
            (153, "Next Track"),
            (160, "Mute"),
            (237, "Media Select"),
        ] {
            assert!(is_known(code));
            assert_eq!(key_label(code, Layout::Qwerty), label);
            assert_eq!(key_label(code, Layout::Azerty), label);
        }
    }

    #[test]
    fn unknown_codes_are_not_truncated_or_aliased() {
        for code in [i32::MIN, -1, 0, 84, 85, 89, 255, 256, 313, 0xe01d, i32::MAX] {
            assert!(!is_known(code));
            for layout in [Layout::Qwerty, Layout::Azerty] {
                assert_eq!(key_label(code, layout), format!("Unknown key ({code})"));
            }
        }
    }

    #[test]
    fn label_table_has_unique_codes_and_nonempty_legends() {
        let mut codes = HashSet::new();
        for &(code, qwerty, azerty) in KEY_LABELS {
            assert!((1..=255).contains(&code));
            assert!(codes.insert(code), "duplicate DIK {code}");
            assert!(!qwerty.is_empty());
            assert!(!azerty.is_empty());
        }
    }

    #[test]
    fn visual_keyboard_has_105_unique_known_keys_and_sensible_widths() {
        let rows = keyboard_rows();
        assert_eq!(rows.len(), 15);
        let mut codes = HashSet::new();
        for row in &rows {
            assert!(!row.is_empty());
            for key in row {
                assert!(is_known(key.code), "unmapped visual key {}", key.code);
                assert!(codes.insert(key.code), "duplicate visual key {}", key.code);
                assert!(key.width.is_finite() && key.width > 0.0);
            }
        }
        assert_eq!(codes.len(), 105);
        for row in &rows[1..6] {
            assert_eq!(row.iter().map(|key| key.width).sum::<f32>(), 15.0);
        }
        assert_eq!(rows[4][0].code, 42);
        assert_eq!(rows[4][1].code, 86);
        assert_eq!(rows[4][2].code, 44);
        assert_eq!(
            rows[5][3],
            Key {
                code: 57,
                width: 6.25
            }
        );
    }

    #[test]
    fn selecting_legends_preserves_physical_codes_and_geometry() {
        let before = keyboard_rows();
        for row in &before {
            for key in row {
                let _ = key_label(key.code, Layout::Qwerty);
                let _ = key_label(key.code, Layout::Azerty);
            }
        }
        assert_eq!(keyboard_rows(), before);
        assert_eq!(before[2][1].code, 16);
        assert_eq!(key_label(before[2][1].code, Layout::Azerty), "A");
        assert_eq!(before[3][1].code, 30);
        assert_eq!(key_label(before[3][1].code, Layout::Azerty), "Q");
    }
}
