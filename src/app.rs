use std::{
    fs::File,
    io::Read,
    path::PathBuf,
    sync::mpsc::{self, Receiver, TryRecvError},
};

use eframe::egui;
use wt_keybind_viewer::bindings::{
    analyzer::{self, Analysis},
    keymap::Layout,
    model::Profile,
};

use crate::ui::{self, actions::ActionListState};

const MAX_IMPORT_BYTES: u64 = 16 * 1024 * 1024;
const EXAMPLE: &str = include_str!("../tests/fixtures/inputs_wt.blk");

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Keyboard,
    Actions,
}

enum ImportSource {
    Dialog,
    Path(PathBuf),
    Example,
}

struct LoadedProfile {
    profile: Profile,
    analysis: Analysis,
    name: String,
    origin: String,
    example: bool,
}

type ImportResult = Result<Option<LoadedProfile>, String>;

pub struct ViewerApp {
    loaded: Option<LoadedProfile>,
    pending: Option<Receiver<ImportResult>>,
    error: Option<String>,
    layout: Layout,
    tab: Tab,
    selected_key: Option<i32>,
    actions: ActionListState,
}

impl ViewerApp {
    pub fn new(cc: &eframe::CreationContext<'_>, initial_path: Option<PathBuf>) -> Self {
        ui::configure(&cc.egui_ctx);
        let mut app = Self {
            loaded: None,
            pending: None,
            error: None,
            layout: Layout::default(),
            tab: Tab::Keyboard,
            selected_key: None,
            actions: ActionListState::default(),
        };
        if let Some(path) = initial_path {
            app.start_import(ImportSource::Path(path), &cc.egui_ctx);
        }
        app
    }

    fn start_import(&mut self, source: ImportSource, ctx: &egui::Context) {
        if self.pending.is_some() {
            return;
        }
        self.error = None;
        let (sender, receiver) = mpsc::channel();
        let ctx = ctx.clone();
        match std::thread::Builder::new()
            .name("profile-import".into())
            .spawn(move || {
                let result = load(source);
                if let Err(error) = &result {
                    tracing::warn!(%error, "Profile import failed");
                }
                let _ = sender.send(result);
                ctx.request_repaint();
            }) {
            Ok(_) => self.pending = Some(receiver),
            Err(error) => self.error = Some(format!("Could not start import worker: {error}")),
        }
    }

    fn poll_import(&mut self) {
        let Some(receiver) = &self.pending else {
            return;
        };
        let result = match receiver.try_recv() {
            Ok(result) => result,
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Disconnected) => {
                Err("Import worker stopped unexpectedly. The current profile was kept.".into())
            }
        };
        self.pending = None;
        match result {
            Ok(Some(loaded)) => {
                tracing::info!(entries = loaded.profile.bindings.len(), "Profile loaded");
                self.loaded = Some(loaded);
                self.selected_key = None;
                self.actions = ActionListState::default();
                self.error = None;
            }
            Ok(None) => {} // A cancelled dialog leaves the current profile intact.
            Err(error) => self.error = Some(error),
        }
    }

    fn header(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                ui.heading("WAR THUNDER");
                ui.label(egui::RichText::new("/ KEYBIND VIEWER").color(ui::MUTED));
                ui.label(egui::RichText::new("READ ONLY").small().color(ui::ACCENT));
            });
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                let available = self.pending.is_none();
                if ui.add_enabled(available, egui::Button::new("Import .blk file…")).clicked() {
                    self.start_import(ImportSource::Dialog, ctx);
                }
                if ui.add_enabled(available, egui::Button::new("Open example")).clicked() {
                    self.start_import(ImportSource::Example, ctx);
                }
                if !available {
                    ui.spinner();
                    ui.label("Opening / importing…");
                }
                ui.separator();
                ui.label("Key legends:");
                ui.selectable_value(&mut self.layout, Layout::Qwerty, "QWERTY");
                ui.selectable_value(&mut self.layout, Layout::Azerty, "AZERTY");
            });
            ui.small("Layout changes display legends only, not physical key codes.");
            if let Some(loaded) = &self.loaded {
                ui.add_space(5.0);
                ui.horizontal_wrapped(|ui| {
                    let prefix = if loaded.example { "Bundled example" } else { "Imported" };
                    ui.strong(format!("{prefix}: {}", loaded.name)).on_hover_text(&loaded.origin);
                    ui.label(format!(
                        "{} entries · {} explicit keyboard codes · {} empty / settings-only entries",
                        loaded.profile.bindings.len(),
                        loaded.analysis.by_key.len(),
                        loaded.profile.bindings.iter().filter(|b| b.inputs.is_empty()).count(),
                    ));
                });
            }
            if let Some(error) = &self.error {
                let mut dismiss = false;
                ui.group(|ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.colored_label(ui::ERROR, "Import failed");
                        dismiss = ui.small_button("Dismiss").clicked();
                    });
                    ui.label(error);
                    if self.loaded.is_some() {
                        ui.small("Your previously loaded profile is still displayed.");
                    }
                });
                if dismiss {
                    self.error = None;
                }
            }
            ui.add_space(5.0);
        });
    }
}

impl eframe::App for ViewerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_import();
        // Also recover visibly if a worker panics before it can request a repaint.
        if self.pending.is_some() {
            ctx.request_repaint_after(std::time::Duration::from_millis(150));
        }
        self.header(ctx);
        egui::CentralPanel::default().show(ctx, |ui| {
            let Some(loaded) = &self.loaded else {
                ui.add_space(40.0);
                ui.heading("Explore your controls, one key at a time.");
                ui.label("Import a War Thunder text .blk controls export to see its explicit bindings.");
                ui.add_space(12.0);
                ui.label("Or choose Open example to inspect the bundled sample — no file is loaded automatically.");
                ui.add_space(12.0);
                ui.colored_label(ui::WARNING, "Base presets and includes are not resolved.");
                ui.label("No explicit binding does not mean a key is free in game.");
                ui.small("Local, read-only inspection. Imports are limited to 16 MiB of UTF-8 text.");
                return;
            };

            ui.colored_label(
                ui::WARNING,
                "Explicit export entries only — no explicit binding ≠ free in game.",
            );
            ui.small("Base presets and includes are unresolved; inherited controls may be absent. Shared keys and potential overlaps are not proven conflicts.");
            if loaded.analysis.overlap_checks_truncated {
                            ui.colored_label(ui::WARNING, "Overlap analysis incomplete: reached the 100,000-pair safety limit. Key usage is complete, but additional duplicates/overlaps may exist.");
                        }
                        ui::profile_metadata(ui, &loaded.profile);
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                ui.selectable_value(&mut self.tab, Tab::Keyboard, "Interactive Keyboard");
                ui.selectable_value(&mut self.tab, Tab::Actions, "Action List");
            });
            ui.separator();
            match self.tab {
                Tab::Keyboard => {
                    if let Some(action) = ui::keyboard::show(
                        ui,
                        &loaded.profile,
                        &loaded.analysis,
                        self.layout,
                        &mut self.selected_key,
                    ) {
                        self.actions.focus_action(action);
                        self.tab = Tab::Actions;
                    }
                }
                Tab::Actions => {
                    self.actions.show(ui, &loaded.profile, &loaded.analysis, self.layout);
                }
            }
        });
    }
}

fn load(source: ImportSource) -> ImportResult {
    let (text, name, origin, example) = match source {
        ImportSource::Example => (
            EXAMPLE.to_owned(),
            "inputs_wt.blk".to_owned(),
            "Bundled example: tests/fixtures/inputs_wt.blk (not an imported user file)".to_owned(),
            true,
        ),
        ImportSource::Dialog => {
            let Some(path) = rfd::FileDialog::new()
                .set_title("Import War Thunder controls export")
                .add_filter("War Thunder controls (.blk)", &["blk"])
                .add_filter("All files", &["*"])
                .pick_file()
            else {
                return Ok(None);
            };
            read_path(path)?
        }
        ImportSource::Path(path) => read_path(path)?,
    };
    let profile =
        analyzer::import(&text).map_err(|error| format!("Cannot parse {name}: {error}"))?;
    let analysis = analyzer::analyze(&profile);
    Ok(Some(LoadedProfile {
        profile,
        analysis,
        name,
        origin,
        example,
    }))
}

fn read_path(path: PathBuf) -> Result<(String, String, String, bool), String> {
    let file =
        File::open(&path).map_err(|error| format!("Cannot open {}: {error}", path.display()))?;
    let text = read_text(file).map_err(|error| format!("{}: {error}", path.display()))?;
    let name = path
        .file_name()
        .unwrap_or(path.as_os_str())
        .to_string_lossy()
        .into_owned();
    Ok((text, name, path.display().to_string(), false))
}

fn read_text(reader: impl Read) -> Result<String, String> {
    // Bound the actual read, not just metadata: a file can grow after a size check.
    let mut bytes = Vec::new();
    reader
        .take(MAX_IMPORT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("Could not read the controls export: {error}"))?;
    if bytes.len() as u64 > MAX_IMPORT_BYTES {
        return Err("File exceeds the 16 MiB import limit. Choose a text controls export.".into());
    }
    String::from_utf8(bytes).map_err(|error| {
        format!(
            "Not valid UTF-8 text (invalid data at byte {}). Export controls as a text .blk file; binary BLK files are not supported.",
            error.utf8_error().valid_up_to(),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_read_accepts_utf8_and_rejects_binary() {
        assert_eq!(read_text("é keyboard".as_bytes()).unwrap(), "é keyboard");
        assert!(read_text(&[0xff, 0xfe][..]).unwrap_err().contains("UTF-8"));
    }

    #[test]
    fn size_limit_is_enforced_on_the_read_itself() {
        assert_eq!(
            read_text(std::io::repeat(b'a').take(MAX_IMPORT_BYTES))
                .unwrap()
                .len() as u64,
            MAX_IMPORT_BYTES,
        );
        assert!(
            read_text(std::io::repeat(b'a'))
                .unwrap_err()
                .contains("16 MiB")
        );
    }

    #[test]
    fn failed_or_cancelled_import_preserves_the_current_profile() {
        for result in [Ok(None), Err("broken file".to_owned())] {
            let (sender, receiver) = mpsc::channel();
            sender.send(result).ok().unwrap();
            let mut app = ViewerApp {
                loaded: Some(load(ImportSource::Example).unwrap().unwrap()),
                pending: Some(receiver),
                error: None,
                layout: Layout::Qwerty,
                tab: Tab::Keyboard,
                selected_key: Some(57),
                actions: ActionListState::default(),
            };
            app.poll_import();
            assert!(app.loaded.as_ref().unwrap().example);
            assert_eq!(app.selected_key, Some(57));
            assert!(app.pending.is_none());
        }
    }

    #[test]
    fn example_is_explicitly_identified() {
        let loaded = load(ImportSource::Example).unwrap().unwrap();
        assert!(loaded.example);
        assert!(!loaded.profile.bindings.is_empty());
        assert!(loaded.origin.contains("not an imported user file"));
    }
}
