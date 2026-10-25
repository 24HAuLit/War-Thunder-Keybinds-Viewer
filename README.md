# War Thunder Keybind Viewer

A local, read-only desktop explorer for War Thunder **text `.blk` controls exports**, built with Rust and egui/eframe. Linux and Windows are the desktop targets. Not affiliated with Gaijin Entertainment.

> **No explicit binding does not mean a key is free in game.** Exported controls can inherit a base preset. This viewer does not load game assets, resolve presets/includes, or invent bindings that are absent from the export.

## Features

- Interactive full-size ISO keyboard with an L-shaped Enter, centered arrow cluster, and double-height numpad keys: green for one explicit entry, orange for multiple entries, gray for none, and blue for the selected key.
- US QWERTY and traditional French AZERTY legends on the same physical scan-code positions, including distinct left/right modifiers, navigation, and numpad keys.
- Click a key to inspect all associated actions and **complete combinations**. Unknown and nonvisual keyboard codes remain selectable.
- English game action names from a bundled, versioned localization catalogue; original identifiers stay visible and unknown commands keep an explicit fallback.
- Action list with text search, device/section/analysis/empty-entry filters, and ascending/descending action or combination sorting.
- Multiple entries for the same action are retained, including empty overrides and exact duplicates.
- Keyboard, mouse buttons, joystick/gamepad buttons, and known axis fields; unfamiliar fields remain inspectable in raw metadata.
- Exact duplicate detection and conservative full-combination overlap notes. Reusing a modifier or key in different combinations is not itself a conflict.
- Clear parse errors with line/column locations. Failed or cancelled imports preserve the currently displayed profile.
- Background importing, a bounded UTF-8 file read, and no changes to the original file. No telemetry or network access in the application.

## Quick start

### NixOS / Nix

With flakes enabled:

```sh
nix develop
cargo run --locked
# Or open a file immediately:
cargo run --locked -- /path/to/inputs_wt.blk
```

`flake.lock` pins nixpkgs. The shell includes stable Rust, Cargo, rustfmt, Clippy, rust-analyzer, pkg-config, Wayland/X11/OpenGL libraries, and Xvfb/xdotool for native smoke testing. It supports `x86_64-linux` and `aarch64-linux`. NixOS users should run the binary from this shell so its dynamically loaded libraries are available.

The Linux file dialog uses the XDG desktop portal: run a session D-Bus service and an appropriate `xdg-desktop-portal` backend (for example, GTK or KDE). These are desktop services, not started by the development shell. If the portal is unavailable, pass a path on the command line. An OpenGL-capable driver is required; headless tests can use Mesa software rendering.

### Other Linux distributions

Install the following build and graphics dependencies. These are **Arch Linux package names**; use your distribution's equivalents:

- `rust` — stable Rust including Cargo; alternatively, install stable Rust with [rustup](https://rustup.rs/).
- `base-devel` — C compiler and linker.
- `pkgconf` — provides `pkg-config`.
- `libglvnd` — OpenGL.
- `libx11`
- `libxcursor`
- `libxi`
- `libxrandr`
- `libxkbcommon`
- `wayland`

For the file picker, also install and configure:

- `dbus`, with a running session bus.
- `xdg-desktop-portal`.
- A suitable portal backend, such as `xdg-desktop-portal-gtk` or `xdg-desktop-portal-kde`.

A working X11 or Wayland session and an OpenGL-capable driver are required to display the GUI (for example, `mesa` for supported Intel/AMD GPUs). If the portal is unavailable, pass the file path on the command line instead:

```sh
cargo run --locked
# Or open an exported .blk controls file immediately, without using the file picker:
# Replace /path/to/inputs_wt.blk with the actual path to your controls file exported from War Thunder.
cargo run --locked -- /path/to/inputs_wt.blk
```

### Windows

For Intel/AMD x64 systems, install [rustup](https://rustup.rs/) and use the stable `x86_64-pc-windows-msvc` Rust toolchain. Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the **Desktop development with C++** workload and ensure these individual components are selected:

- **MSVC C++ x64/x86 build tools** (named **MSVC v143 - VS 2022 C++ x64/x86 build tools** in Visual Studio 2022).
- **Windows 11 SDK**, for example version **10.0.26100.0**. This SDK also supports targeting Windows 10; its name does not mean Windows 11 is required to run the application.

The full Visual Studio IDE is not required. An existing Visual Studio or Build Tools installation with these components is sufficient. From PowerShell:

```powershell
rustup default stable-x86_64-pc-windows-msvc
cargo run --locked
# Or open an exported .blk controls file immediately:
cargo run --locked -- "C:\path\to\inputs_wt.blk"
```

Replace `C:\path\to\inputs_wt.blk` with the actual path to your `.blk` controls file exported from War Thunder.

Nix is not required. The Windows native file dialog is supplied by `rfd`. Build a distributable executable with `cargo build --release --locked`; it is placed at `target/release/wt-keybind-viewer.exe` on Windows or `target/release/wt-keybind-viewer` on Linux.

## Usage

1. Export your controls from War Thunder to a text `.blk` file.
2. Choose **Import .blk file…**, or provide its path as the first command-line argument.
3. Select **QWERTY** or **AZERTY** for the legends you want to read. This never changes the imported codes or interprets the current OS keyboard layout.
4. In **Interactive Keyboard**, click a key to view every containing combination, source entry, and analysis note. Scroll horizontally on smaller windows.
5. In **Action List**, search English action names, original identifiers, fallback names, combinations, devices, or contexts. Filter and sort without collapsing repeated entries.
6. Expand **Import notes & source metadata** for unresolved preset paths, device mappings, unknown fields, and the ordered document tree.

**Open example** explicitly loads the bundled real export at `tests/fixtures/inputs_wt.blk`. It is labeled as an example, not as your imported configuration. Nothing is loaded automatically unless a path is supplied.

Use `RUST_LOG=debug cargo run --locked` for verbose diagnostics on Linux (PowerShell: `$env:RUST_LOG = "debug"`). Log messages contain load/error diagnostics, not uploaded data.

## Architecture

```text
src/
├── lib.rs                GUI-independent public core
├── blk/
│   ├── mod.rs            Ordered Block / Entry / Property model
│   └── parser.rs         Text parser and located errors
├── bindings/
│   ├── model.rs          Explicit bindings, input types, profile, analysis notes
│   ├── keymap.rs         Physical scan codes, layout legends, keyboard geometry
│   ├── analyzer.rs       Import/extraction, key index, conservative overlap analysis
│   └── localization/     Pinned English game labels and exact section-aware lookup
├── main.rs               Native startup and logging
├── app.rs                Application state and background file import
└── ui/
    ├── mod.rs            Theme, raw metadata, shared binding details
    ├── keyboard.rs       Interactive keyboard and selected-key details
    └── actions.rs        Action search, filters, and sorting
```

The core accepts a string and performs no filesystem or GUI operations:

```rust
use wt_keybind_viewer::bindings::analyzer::{analyze, import};

let profile = import("controls { hotkeys { ID_TEST { keyboardKey:i=57 } } }")
    .expect("valid controls export");
let analysis = analyze(&profile);
assert_eq!(analysis.indices_for_key(57), &[0]);
```

`cargo test --no-default-features --locked` builds/tests the core without eframe, rfd, or desktop logging dependencies. This separation permits a future browser frontend; no WebAssembly frontend is implemented.

### Text parser decision

Before implementing the parser, the following existing Rust options were investigated:

- [`wt_blk`](https://github.com/Warthunder-Open-Source-Foundation/wt_blk): intended for internal **binary** BLK/VROMF assets; not a direct text-export loader.
- [Gaijin's `tree-sitter-blk`](https://github.com/GaijinEntertainment/tree-sitter-blk): an official text grammar, but introduces a C parser/tree-sitter runtime and restricts recognized type names. Its syntax was consulted; the viewer needs a small ordered value model that also retains unknown typed fields.

The MVP therefore uses a custom recursive-descent **text-export subset**, not a complete replacement for Dagor's DataBlock loader. The document retains ordered blocks/properties, repeated names, unescaped string values, raw unknown types, and vector scalar text. Comments, whitespace, and original quoting are not retained, so this model must not be used to round-trip/edit files. Strings support Dagor tilde escapes and conventional backslash escapes; unknown escapes are preserved.

Known scalar types are validated (`i`, `i64`, finite `r`, and `b`). Includes are preserved as references and never followed. Malformed or unsupported syntax produces a located error rather than a partially imported profile.

### Keyboard mapping evidence

[Gaijin's Dagor keyboard IDs](https://github.com/GaijinEntertainment/DagorEngine/blob/main/prog/dagorInclude/drv/hid/dag_hiKeybIds.h) use the same physical offsets as [Microsoft's DIK constants](https://github.com/microsoft/win32metadata/blob/main/generation/WinSDK/RecompiledIdlHeaders/um/dinput.h). Representative values in the real export include:

| Export code | Physical key |
| --- | --- |
| 29 / `0x1d` | Left Ctrl |
| 42 / `0x2a` | Left Shift |
| 56 / `0x38` | Left Alt |
| 57 / `0x39` | Space |
| 183 / `0xb7` | Print Screen |
| 203 / `0xcb` | Left arrow |
| 205 / `0xcd` | Right arrow |

These are **not** Windows virtual-key numbers, USB HID usages, or character codes. For example, code 16 is Q on US QWERTY and A on French AZERTY. Unknown signed codes are preserved without truncation or remapping.

### English action names

Action cards in both views display English game labels while retaining the original `.blk` identifiers. Search accepts either. A bundled catalogue contains 1,659 nonempty localization entries from the game's `controls.csv` and `inf.csv`, pinned to a specific community datamine revision. Hotkeys and axes use their respective localization namespaces, not guessed aliases.

Unknown identifiers retain an identifier-based fallback, marked in the tooltip. In the real bundled export, 81 of 97 entries have game labels, including every axis; 16 drone range-key entries have no exact translation in the source catalogues. The snapshot does not resolve presets or add commands/bindings absent from an export.

The app and build remain entirely offline, with no new Rust dependencies. See [catalogue provenance, attribution, and regeneration instructions](src/bindings/localization/README.md). Python is only needed to regenerate the catalogue, not to use or build the viewer.

### Binding and conflict semantics

- Each action block under `controls/hotkeys` or `controls/axes` is retained as a separate entry. Repeated input properties within one entry form one combination; repeated action blocks are alternatives.
- Empty entries are shown as empty overrides/settings-only, not assumed to disable every inherited control. Their game-specific merge behavior is not simulated.
- The original ordered document remains available alongside extracted entries, including settings, activation modes, device metadata, and unfamiliar input properties. Recognizable unfamiliar key/button/axis fields are labeled unknown, not interpreted as known devices.
- Category is the explicit `Hotkeys`/`Axes` section. Nested structural groups provide a context label; no vehicle/game context is guessed from action names. The supplied flat export declares **no such contexts**.
- Exact duplicate notes require the same action, section, context, full input combination, and ordered direct properties. Entries with nested blocks are excluded from exact-duplicate assertions because their direct properties alone do not establish identity.
- Other identical full combinations are potential same-context overlaps or **context unknown** notes, never confirmed game conflicts. Different known structural contexts are not flagged against one another.
- Disabled negative axis IDs and unknown input encodings do not participate in overlap comparisons. Joystick indices are displayed as exported; device offsets remain in metadata rather than being guessed into controller button names.

## Tests and development

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked
cargo test --locked --no-default-features
cargo build --locked
```

Tests cover parser syntax/diagnostics, malformed input, nesting limits, repeated entries, physical codes/layouts, combinations, conservative analysis, import resource limits, search/filter/sort behavior, failed-import state preservation, and both UI views at narrow/wide sizes. Integration tests use the **unaltered real export**: 92 hotkey entries, 5 axis entries, 80 keyboard properties, and 46 distinct keyboard codes. The screenshot duplicates and Space reuse are explicit regression cases.

Linux native startup has also been smoke-tested with Xvfb/Mesa and the real file, successfully loading all 97 entries. This does not substitute for manual testing on every desktop/driver. Windows build and native dialog behavior have not been verified locally; the CI workflow runs formatting, strict Clippy, tests, and builds on Linux and Windows when pushed.

## Known limitations

- Base presets, includes, game defaults, live game localization assets, activation semantics, and the game's final merge/context rules are not resolved. There are no guaranteed “free key” or definitive conflict claims.
- English action names use a versioned game-localization snapshot, not a live or exhaustive catalogue. Missing translations use identifier-based fallbacks; no other display language is implemented. Mouse/controller indices and axis properties are displayed rather than visualized or fully decoded.
- Input is UTF-8 **text BLK**, not binary/compressed BLK or VROMF. Bracketed arrays/matrices, triple-quoted strings, and hexadecimal integer literals are not supported. Unknown property types and ordinary vector scalar text are preserved, but not semantically validated.
- Resource safeguards: desktop imports are at most **16 MiB**, parser nesting is at most **128**, derived context/source paths share a **64 MiB allocation budget**, and overlap analysis checks at most **100,000 pairs**. Exceeding the pair limit leaves key usage complete but shows a prominent incomplete-analysis warning. Exceeding other limits rejects the import without replacing the current profile.
- QWERTY and AZERTY are display legends on the same 105-key ISO geometry, not comprehensive national/OS input-method support.
- No editing, exporting, profile management, browser UI, or comparisons are implemented.

## License

Project code is MIT; see [LICENSE](LICENSE). The bundled export is a user-provided regression fixture. Bundled game localization strings originate from Gaijin Entertainment and are not relicensed under MIT; see [source attribution](src/bindings/localization/README.md). War Thunder and related marks belong to their respective owners.
