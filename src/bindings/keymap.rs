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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyShape {
    Rectangle,
    /// The lower half is inset by 0.25 key pitch units on the left.
    IsoEnter,
}

/// Physical key bounds in key pitch units, with the origin at the top left.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Key {
    pub code: i32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub shape: KeyShape,
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

/// A full-size 105-key ISO keyboard, independent of the chosen legends.
/// Positions and bounds are in key pitch units; ISO Enter's bounds include its notch.
pub fn keyboard_keys() -> Vec<Key> {
    fn key(code: i32, x: f32, y: f32) -> Key {
        Key {
            code,
            x,
            y,
            width: 1.0,
            height: 1.0,
            shape: KeyShape::Rectangle,
        }
    }

    vec![
        // Function row.
        key(1, 0.0, 0.0),
        key(59, 2.0, 0.0),
        key(60, 3.0, 0.0),
        key(61, 4.0, 0.0),
        key(62, 5.0, 0.0),
        key(63, 6.5, 0.0),
        key(64, 7.5, 0.0),
        key(65, 8.5, 0.0),
        key(66, 9.5, 0.0),
        key(67, 11.0, 0.0),
        key(68, 12.0, 0.0),
        key(87, 13.0, 0.0),
        key(88, 14.0, 0.0),
        // Main number row.
        key(41, 0.0, 1.5),
        key(2, 1.0, 1.5),
        key(3, 2.0, 1.5),
        key(4, 3.0, 1.5),
        key(5, 4.0, 1.5),
        key(6, 5.0, 1.5),
        key(7, 6.0, 1.5),
        key(8, 7.0, 1.5),
        key(9, 8.0, 1.5),
        key(10, 9.0, 1.5),
        key(11, 10.0, 1.5),
        key(12, 11.0, 1.5),
        key(13, 12.0, 1.5),
        Key {
            width: 2.0,
            ..key(14, 13.0, 1.5)
        },
        // Tab and the upper part of ISO Enter.
        Key {
            width: 1.5,
            ..key(15, 0.0, 2.5)
        },
        key(16, 1.5, 2.5),
        key(17, 2.5, 2.5),
        key(18, 3.5, 2.5),
        key(19, 4.5, 2.5),
        key(20, 5.5, 2.5),
        key(21, 6.5, 2.5),
        key(22, 7.5, 2.5),
        key(23, 8.5, 2.5),
        key(24, 9.5, 2.5),
        key(25, 10.5, 2.5),
        key(26, 11.5, 2.5),
        key(27, 12.5, 2.5),
        Key {
            width: 1.5,
            height: 2.0,
            shape: KeyShape::IsoEnter,
            ..key(28, 13.5, 2.5)
        },
        // Caps row fits into ISO Enter's lower-left notch.
        Key {
            width: 1.75,
            ..key(58, 0.0, 3.5)
        },
        key(30, 1.75, 3.5),
        key(31, 2.75, 3.5),
        key(32, 3.75, 3.5),
        key(33, 4.75, 3.5),
        key(34, 5.75, 3.5),
        key(35, 6.75, 3.5),
        key(36, 7.75, 3.5),
        key(37, 8.75, 3.5),
        key(38, 9.75, 3.5),
        key(39, 10.75, 3.5),
        key(40, 11.75, 3.5),
        key(43, 12.75, 3.5),
        Key {
            width: 1.25,
            ..key(42, 0.0, 4.5)
        },
        key(86, 1.25, 4.5),
        key(44, 2.25, 4.5),
        key(45, 3.25, 4.5),
        key(46, 4.25, 4.5),
        key(47, 5.25, 4.5),
        key(48, 6.25, 4.5),
        key(49, 7.25, 4.5),
        key(50, 8.25, 4.5),
        key(51, 9.25, 4.5),
        key(52, 10.25, 4.5),
        key(53, 11.25, 4.5),
        Key {
            width: 2.75,
            ..key(54, 12.25, 4.5)
        },
        Key {
            width: 1.25,
            ..key(29, 0.0, 5.5)
        },
        Key {
            width: 1.25,
            ..key(219, 1.25, 5.5)
        },
        Key {
            width: 1.25,
            ..key(56, 2.5, 5.5)
        },
        Key {
            width: 6.25,
            ..key(57, 3.75, 5.5)
        },
        Key {
            width: 1.25,
            ..key(184, 10.0, 5.5)
        },
        Key {
            width: 1.25,
            ..key(220, 11.25, 5.5)
        },
        Key {
            width: 1.25,
            ..key(221, 12.5, 5.5)
        },
        Key {
            width: 1.25,
            ..key(157, 13.75, 5.5)
        },
        // Navigation and inverted-T arrows.
        key(183, 15.5, 0.0),
        key(70, 16.5, 0.0),
        key(197, 17.5, 0.0),
        key(210, 15.5, 1.5),
        key(199, 16.5, 1.5),
        key(201, 17.5, 1.5),
        key(211, 15.5, 2.5),
        key(207, 16.5, 2.5),
        key(209, 17.5, 2.5),
        key(200, 16.5, 4.5),
        key(203, 15.5, 5.5),
        key(208, 16.5, 5.5),
        key(205, 17.5, 5.5),
        // Numpad: tall keys are emitted only once.
        key(69, 19.0, 1.5),
        key(181, 20.0, 1.5),
        key(55, 21.0, 1.5),
        key(74, 22.0, 1.5),
        key(71, 19.0, 2.5),
        key(72, 20.0, 2.5),
        key(73, 21.0, 2.5),
        Key {
            height: 2.0,
            ..key(78, 22.0, 2.5)
        },
        key(75, 19.0, 3.5),
        key(76, 20.0, 3.5),
        key(77, 21.0, 3.5),
        key(79, 19.0, 4.5),
        key(80, 20.0, 4.5),
        key(81, 21.0, 4.5),
        Key {
            height: 2.0,
            ..key(156, 22.0, 4.5)
        },
        Key {
            width: 2.0,
            ..key(82, 19.0, 5.5)
        },
        key(83, 21.0, 5.5),
    ]
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

    fn find_key(keys: &[Key], code: i32) -> Key {
        *keys
            .iter()
            .find(|key| key.code == code)
            .expect("missing key")
    }

    // Split ISO Enter into its actual occupied areas, not its bounding box.
    fn occupied_rectangles(key: Key) -> Vec<(f32, f32, f32, f32)> {
        match key.shape {
            KeyShape::Rectangle => vec![(key.x, key.y, key.width, key.height)],
            KeyShape::IsoEnter => vec![
                (key.x, key.y, key.width, 1.0),
                (
                    key.x + 0.25,
                    key.y + 1.0,
                    key.width - 0.25,
                    key.height - 1.0,
                ),
            ],
        }
    }

    #[test]
    fn visual_keyboard_has_105_unique_known_keys_and_valid_bounds() {
        let keys = keyboard_keys();
        assert_eq!(keys.len(), 105);
        let mut codes = HashSet::new();
        for key in &keys {
            assert!(is_known(key.code), "unmapped visual key {}", key.code);
            assert!(codes.insert(key.code), "duplicate visual key {}", key.code);
            assert!(key.x.is_finite() && key.x >= 0.0);
            assert!(key.y.is_finite() && key.y >= 0.0);
            assert!(key.width.is_finite() && key.width > 0.0);
            assert!(key.height.is_finite() && key.height > 0.0);
            assert!(key.x + key.width <= 23.0);
            assert!(key.y + key.height <= 6.5);
            assert_eq!(
                key.height,
                if matches!(key.code, 28 | 78 | 156) {
                    2.0
                } else {
                    1.0
                }
            );
            assert_eq!(
                key.shape,
                if key.code == 28 {
                    KeyShape::IsoEnter
                } else {
                    KeyShape::Rectangle
                }
            );
            if !matches!(
                key.code,
                14 | 15 | 28 | 29 | 42 | 54 | 56 | 57 | 58 | 82 | 157 | 184 | 219 | 220 | 221
            ) {
                assert_eq!(key.width, 1.0, "DIK {}", key.code);
            }
        }
        assert_eq!(codes.len(), 105);
    }

    #[test]
    fn function_row_has_explicit_group_gaps() {
        let keys = keyboard_keys();
        let expected = [
            (1, 0.0),
            (59, 2.0),
            (60, 3.0),
            (61, 4.0),
            (62, 5.0),
            (63, 6.5),
            (64, 7.5),
            (65, 8.5),
            (66, 9.5),
            (67, 11.0),
            (68, 12.0),
            (87, 13.0),
            (88, 14.0),
        ];
        for (code, x) in expected {
            let key = find_key(&keys, code);
            assert_eq!((key.x, key.y, key.width, key.height), (x, 0.0, 1.0, 1.0));
        }
    }

    #[test]
    fn main_rows_tile_exactly_from_zero_to_fifteen() {
        let keys = keyboard_keys();
        for y in [1.5, 2.5, 3.5, 4.5, 5.5] {
            let mut row: Vec<_> = keys
                .iter()
                .copied()
                .filter(|key| key.x < 15.0)
                .flat_map(occupied_rectangles)
                .filter(|rect| rect.1 == y)
                .collect();
            row.sort_by(|a, b| a.0.total_cmp(&b.0));
            assert!(!row.is_empty());
            let mut right = 0.0;
            for (x, _, width, height) in row {
                assert_eq!(x, right, "gap or overlap at y={y}, x={x}");
                assert_eq!(height, 1.0);
                right = x + width;
            }
            assert_eq!(right, 15.0, "main row at y={y}");
        }
    }

    #[test]
    fn iso_enter_and_staggered_main_rows_are_positioned_correctly() {
        let keys = keyboard_keys();
        assert_eq!(
            find_key(&keys, 28),
            Key {
                code: 28,
                x: 13.5,
                y: 2.5,
                width: 1.5,
                height: 2.0,
                shape: KeyShape::IsoEnter,
            }
        );
        assert_eq!(
            occupied_rectangles(find_key(&keys, 28)),
            vec![(13.5, 2.5, 1.5, 1.0), (13.75, 3.5, 1.25, 1.0),]
        );
        for (code, x, y, width) in [
            (41, 0.0, 1.5, 1.0),
            (14, 13.0, 1.5, 2.0),
            (15, 0.0, 2.5, 1.5),
            (58, 0.0, 3.5, 1.75),
            (43, 12.75, 3.5, 1.0),
            (42, 0.0, 4.5, 1.25),
            (86, 1.25, 4.5, 1.0),
            (54, 12.25, 4.5, 2.75),
            (29, 0.0, 5.5, 1.25),
            (219, 1.25, 5.5, 1.25),
            (56, 2.5, 5.5, 1.25),
            (57, 3.75, 5.5, 6.25),
            (184, 10.0, 5.5, 1.25),
            (220, 11.25, 5.5, 1.25),
            (221, 12.5, 5.5, 1.25),
            (157, 13.75, 5.5, 1.25),
        ] {
            let key = find_key(&keys, code);
            assert_eq!(
                (key.x, key.y, key.width, key.height),
                (x, y, width, 1.0),
                "DIK {code}"
            );
        }
        for (codes, x, y) in [
            (2..=13, 1.0, 1.5),
            (16..=27, 1.5, 2.5),
            (30..=40, 1.75, 3.5),
            (44..=53, 2.25, 4.5),
        ] {
            for (offset, code) in codes.enumerate() {
                let key = find_key(&keys, code);
                assert_eq!(
                    (key.x, key.y, key.width),
                    (x + offset as f32, y, 1.0),
                    "DIK {code}"
                );
            }
        }
    }

    #[test]
    fn navigation_and_arrows_align_with_main_rows() {
        let keys = keyboard_keys();
        for (codes, y) in [
            ([183, 70, 197], 0.0),
            ([210, 199, 201], 1.5),
            ([211, 207, 209], 2.5),
            ([203, 208, 205], 5.5),
        ] {
            for (column, code) in codes.into_iter().enumerate() {
                let key = find_key(&keys, code);
                assert_eq!((key.x, key.y), (15.5 + column as f32, y), "DIK {code}");
            }
        }
        let up = find_key(&keys, 200);
        let down = find_key(&keys, 208);
        assert_eq!((up.x, up.y), (16.5, 4.5));
        assert_eq!((up.x, up.y + up.height), (down.x, down.y));
    }

    #[test]
    fn numpad_has_aligned_columns_tall_keys_and_wide_zero() {
        let keys = keyboard_keys();
        for (codes, y) in [
            (&[69, 181, 55, 74][..], 1.5),
            (&[71, 72, 73, 78][..], 2.5),
            (&[75, 76, 77][..], 3.5),
            (&[79, 80, 81, 156][..], 4.5),
        ] {
            for (column, &code) in codes.iter().enumerate() {
                let key = find_key(&keys, code);
                assert_eq!(
                    (key.x, key.y, key.width),
                    (19.0 + column as f32, y, 1.0),
                    "DIK {code}"
                );
            }
        }
        let plus = find_key(&keys, 78);
        let enter = find_key(&keys, 156);
        assert_eq!((plus.height, enter.height), (2.0, 2.0));
        assert_eq!((plus.x, plus.y + plus.height), (enter.x, enter.y));
        assert_eq!(enter.y + enter.height, 6.5);
        let zero = find_key(&keys, 82);
        let decimal = find_key(&keys, 83);
        assert_eq!(
            (zero.x, zero.y, zero.width, zero.height),
            (19.0, 5.5, 2.0, 1.0)
        );
        assert_eq!((decimal.x, decimal.y), (21.0, 5.5));
        assert_eq!(zero.x + zero.width, decimal.x);
        assert_eq!(decimal.x + decimal.width, enter.x);
    }

    #[test]
    fn physical_key_shapes_do_not_overlap() {
        let keys = keyboard_keys();
        for (index, &a) in keys.iter().enumerate() {
            for &b in &keys[index + 1..] {
                for (ax, ay, aw, ah) in occupied_rectangles(a) {
                    for (bx, by, bw, bh) in occupied_rectangles(b) {
                        let overlaps = ax < bx + bw && bx < ax + aw && ay < by + bh && by < ay + ah;
                        assert!(!overlaps, "DIK {} overlaps DIK {}", a.code, b.code);
                    }
                }
            }
        }
    }

    #[test]
    fn selecting_legends_preserves_physical_codes_and_geometry() {
        let before = keyboard_keys();
        for key in &before {
            let _ = key_label(key.code, Layout::Qwerty);
            let _ = key_label(key.code, Layout::Azerty);
        }
        assert_eq!(keyboard_keys(), before);
        assert_eq!(key_label(find_key(&before, 16).code, Layout::Azerty), "A");
        assert_eq!(key_label(find_key(&before, 30).code, Layout::Azerty), "Q");
    }
}
