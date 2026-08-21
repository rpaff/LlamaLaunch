#![windows_subsystem = "windows"]

use std::process::Command;
use std::os::windows::process::CommandExt;

use eframe::egui;

mod config;
mod models;
mod ui;

/// Parse a shell-style arguments string into separate args, respecting quotes.
/// This fixes the "invalid argument" bug where split_whitespace() would break
/// paths with spaces (e.g. `"--model C:/path/to/my model.gguf"`).
fn parse_args(raw: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in raw.chars() {
        match ch {
            ' ' | '\t' | '\n' | '\r' => {
                if !current.is_empty() {
                    if in_quotes {
                        current.push(ch);
                    } else {
                        args.push(current.clone());
                        current.clear();
                    }
                }
                // if in quotes, we skip the whitespace (it's inside)
            }
            '"' => {
                if in_quotes {
                    // closing quote
                    in_quotes = false;
                    args.push(current.clone());
                    current.clear();
                } else if !current.is_empty() {
                    // opening quote — keep what we have so far, start quoting
                    in_quotes = true;
                } else {
                    // opening quote at start of arg
                    in_quotes = true;
                }
            }
            c => {
                current.push(c);
            }
        }
    }
    if !current.is_empty() {
        args.push(current);
    }
    args
}

/// Form data for adding a new model inline.
struct AddModelForm {
    name: String,
    description: String,
    args: String,
}

impl AddModelForm {
    fn new() -> Self {
        AddModelForm {
            name: String::new(),
            description: String::new(),
            args: String::new(),
        }
    }
}

/// Form data for editing an existing model.
struct EditModelForm {
    name: String,
    description: String,
    args: String,
}

impl EditModelForm {
    fn from_model(m: &models::ModelConfig) -> Self {
        EditModelForm {
            name: m.name.clone(),
            description: m.description.clone(),
            args: m.args.clone(),
        }
    }
}

/// UI state for settings — isolated so it can be mutably borrowed independently.
struct SettingsState {
    llama_server_path: String,
    models: Vec<models::ModelConfig>,
    add_model_form: Option<AddModelForm>,
    edit_model_form: Option<(usize, EditModelForm)>,
}

impl SettingsState {
    fn from_config(config: &config::Config) -> Self {
        SettingsState {
            llama_server_path: config.llama_server_path.clone(),
            models: config.models.clone(),
            add_model_form: None,
            edit_model_form: None,
        }
    }

    fn to_config(&self) -> config::Config {
        config::Config {
            llama_server_path: self.llama_server_path.clone(),
            models: self.models.clone(),
        }
    }
}

/// Top-level app state.
struct LlamaServerManagerApp {
    config: config::Config,
    settings: SettingsState,
    active_model_id: Option<String>,
    running_model: Option<models::RunningModel>,
    log_stream: Option<ui::logs::ModelLogStream>,
    model_logs: Vec<String>,
    auto_scroll: bool,
    open_settings: bool,
    stop_requested: bool,
}

impl LlamaServerManagerApp {
    fn new() -> Self {
        let config = config::Config::load().unwrap_or_else(|e| {
            eprintln!("Error loading config: {}", e);
            config::Config::default()
        });
        Self {
            config: config.clone(),
            settings: SettingsState::from_config(&config),
            active_model_id: None,
            running_model: None,
            log_stream: None,
            model_logs: Vec::new(),
            auto_scroll: true,
            open_settings: false,
            stop_requested: false,
        }
    }

    fn stop_current_model(&mut self) {
        if let Some(mut running) = self.running_model.take() {
            let _ = running.stop();
            if let Some(mut ls) = self.log_stream.take() {
                ls.finish();
            }
        }
        self.active_model_id = None;
        self.model_logs.clear();
        self.stop_requested = false;
    }

    fn start_model(&mut self, model_id: &str) {
        if self.running_model.is_some() {
            eprintln!("Stop the current model first.");
            return;
        }
        let model_config = self
            .settings
            .models
            .iter()
            .find(|m| m.id == model_id)
            .cloned();
        let Some(model_config) = model_config else {
            eprintln!("Model {} not found.", model_id);
            return;
        };
        let server_path = match self.config.resolved_server_path() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("Error resolving server path: {}", e);
                return;
            }
        };
        if !server_path.exists() {
            eprintln!("llama-server.exe not found: {}", server_path.display());
            return;
        }
        let args = parse_args(&model_config.args);
        if args.is_empty() {
            eprintln!("Model '{}' has no args.", model_config.name);
            return;
        }
        // FIX: set working directory to where llama-server.exe lives so it finds
        // its config files / DLLs / models relative to itself — this is what
        // makes console-launched behavior match our launcher.
        let server_dir = server_path.parent().map(std::path::Path::to_path_buf);
        // FIX: CREATE_NO_WINDOW (0x08000000) prevents a CMD console from appearing.
        // windows_subsystem = "windows" only affects this process, not children.
        let mut child = match Command::new(&server_path)
            .args(&args)
            .current_dir(server_dir.as_ref().unwrap_or(&server_path))
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .creation_flags(0x08000000)
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to start llama-server: {}", e);
                return;
            }
        };
        let log_stream = ui::logs::ModelLogStream::new(&mut child);
        self.running_model = Some(models::RunningModel {
            config: model_config,
            child,
        });
        self.active_model_id = Some(model_id.to_string());
        self.log_stream = Some(log_stream);
        self.model_logs.clear();
    }

    fn update_logs(&mut self) {
        if let Some(ref mut log_stream) = self.log_stream {
            self.model_logs.extend(log_stream.poll(100));
        }
        if let Some(mut running) = self.running_model.take() {
            match running.child.try_wait() {
                Ok(Some(_)) => {
                    self.running_model = None;
                    self.active_model_id = None;
                    if let Some(mut ls) = self.log_stream.take() {
                        ls.finish();
                    }
                }
                Ok(None) => {
                    self.running_model = Some(running);
                }
                Err(e) => {
                    eprintln!("Process check error: {}", e);
                    self.running_model = None;
                    self.active_model_id = None;
                    if let Some(mut ls) = self.log_stream.take() {
                        ls.finish();
                    }
                }
            }
        }
    }

    fn save_config(&mut self) {
        if let Err(e) = self.config.save() {
            eprintln!("Error saving config: {}", e);
        }
    }

    fn draw_home(&mut self, ctx: &egui::Context) {
        // --- Top panel ---
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Llama Launch");
                ui.separator();
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Settings").clicked() {
                        self.open_settings = true;
                    }
                });
            });
        });

        // --- Model cards ---
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.scope(|ui| {
                ui.visuals_mut().override_text_color = Some(egui::Color32::from_black_alpha(255));
                ui.visuals_mut().panel_fill = egui::Color32::from_rgb(245, 245, 245);
                ui.visuals_mut().widgets.noninteractive.bg_stroke =
                    egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(204, 204, 204));
                ui.visuals_mut().widgets.inactive.fg_stroke =
                    egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(255, 255, 255));

                let avail = ui.available_width();
                let card_w = (avail - 60.0) / 3.0;

                // Collect card data first to avoid borrow conflicts
                let cards: Vec<(String, String, bool, String)> = self
                    .settings
                    .models
                    .iter()
                    .map(|m| {
                        let is_active = self.active_model_id.as_deref() == Some(&m.id);
                        (m.name.clone(), m.description.clone(), is_active, m.id.clone())
                    })
                    .collect();

                for (name, desc, is_active, model_id) in &cards {
                    ui.horizontal(|ui| {
                        let color = if *is_active {
                            egui::Color32::from_rgb(0, 170, 85)
                        } else {
                            egui::Color32::from_rgb(50, 50, 50)
                        };
                        ui.add_space(20.0);
                        let frame = egui::Frame::none()
                            .fill(color)
                            .stroke(egui::Stroke::new(
                                2.0_f32,
                                if *is_active {
                                    egui::Color32::from_rgb(0, 200, 100)
                                } else {
                                    egui::Color32::from_rgb(204, 204, 204)
                                },
                            ))
                            .inner_margin(egui::Margin::symmetric(15.0, 15.0));
                        frame.show(ui, |ui| {
                            ui.set_max_width(card_w);
                            ui.label(
                                egui::RichText::new(name)
                                    .size(16.0)
                                    .strong()
                                    .color(if *is_active {
                                        egui::Color32::WHITE
                                    } else {
                                        egui::Color32::from_rgb(255, 255, 255)
                                    }),
                            );
                            if !desc.is_empty() {
                                ui.label(
                                    egui::RichText::new(desc)
                                        .size(12.0)
                                        .color(if *is_active {
                                            egui::Color32::from_rgb(220, 220, 220)
                                        } else {
                                            egui::Color32::from_rgb(200, 200, 200)
                                        }),
                                );
                            }
                            ui.separator();
                            if *is_active {
                                if ui.button("Stop").clicked() {
                                    self.stop_requested = true;
                                }
                            } else {
                                if ui.button("Start").clicked() {
                                    self.start_model(model_id);
                                }
                            }
                        });
                        ui.add_space(15.0);
                    });
                    ui.add_space(15.0);
                }
            });
        });

        // --- Logs panel (show whenever a model is running, even if logs haven't arrived yet) ---
        if self.running_model.is_some() {
            egui::TopBottomPanel::bottom("logs_panel")
                .resizable(true)
                .show(ctx, |ui| {
                    ui.scope(|ui| {
                        ui.visuals_mut().override_text_color =
                            Some(egui::Color32::from_black_alpha(255));
                        ui.visuals_mut().panel_fill = egui::Color32::from_rgb(30, 30, 30);
                        ui.horizontal(|ui| {
                            if self.running_model.is_some() {
                                ui.label(
                                    egui::RichText::new("Logs — loading...").strong()
                                        .color(egui::Color32::YELLOW),
                                );
                            } else {
                                ui.label(egui::RichText::new("Logs").strong());
                            }
                            ui.separator();
                            if ui.button("Clear").clicked() {
                                self.model_logs.clear();
                            }
                            ui.separator();
                            ui.checkbox(&mut self.auto_scroll, "Auto-scroll");
                            ui.separator();
                            if ui.button("Stop").clicked() {
                                self.stop_requested = true;
                            }
                        });
                        ui.separator();
                        let lines: Vec<String> =
                            self.model_logs.iter().take(200).cloned().collect();
                        if lines.is_empty() {
                            ui.monospace("Waiting for model output...\n");
                        } else {
                            egui::ScrollArea::vertical()
                                .auto_shrink(false)
                                .stick_to_bottom(self.auto_scroll)
                                .show(ui, |ui| {
                                    ui.monospace(lines.join("\n"));
                                });
                        }
                    });
                });
        }
    }

    fn draw_settings(&mut self, ctx: &egui::Context) {
        if !self.open_settings {
            return;
        }
        // NON-resizable window with fixed size — the content scrolls internally.
        // This is what stops the dialog from stretching when the user types
        // long launch arguments into the Args field.
        egui::Window::new("Settings")
            .open(&mut self.open_settings)
            .resizable(false)
            .default_pos([50.0, 50.0])
            .default_size([520.0, 640.0])
            .max_size([530.0, 680.0])
            .show(ctx, |ui| {
                // Force the entire settings body to never exceed window width,
                // preventing Start Args from stretching the dialog wider than
                // the Settings window itself.
                ui.scope(|ui| {
                    let max_w = ui.max_rect().width().min(520.0);
                    ui.set_max_width(max_w);

                    egui::ScrollArea::vertical()
                        .vscroll(true)
                        .show(ui, |ui| {
                            ui_settings_body(ui, &mut self.settings, &mut self.config);
                        });
                });
            });
    }
}

fn ui_settings_body(
    ui: &mut egui::Ui,
    s: &mut SettingsState,
    config: &mut config::Config,
) {
    // Helper: render a single-line field that fills the remaining row width
    // but can never grow beyond it. This is what stops the dialog from
    // stretching while the user types long launch arguments.
    fn bounded_singleline(ui: &mut egui::Ui, text: &mut String) {
        let w = ui.available_width().max(120.0);
        ui.scope(|ui| {
            ui.set_max_width(w);
            ui.add(egui::TextEdit::singleline(text).desired_width(w));
        });
    }

    ui.add_space(20.0);
    ui.label(egui::RichText::new("llama-server.exe path").strong());
    ui.horizontal(|ui| {
        let w = (ui.available_width() - 90.0).max(120.0);
        ui.scope(|ui| {
            ui.set_max_width(w);
            ui.add(egui::TextEdit::singleline(&mut s.llama_server_path).desired_width(w));
        });
        if ui.button("Browse").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("LLama Server", &["exe"])
                .pick_file()
            {
                s.llama_server_path = path.to_string_lossy().to_string();
            }
        }
    });
    ui.label(egui::RichText::new("Path to llama-server.exe").small());
    ui.add_space(10.0);
    ui.separator();
    ui.add_space(10.0);
    ui.label(egui::RichText::new("Models").strong());

    if s.models.is_empty() {
        ui.label(egui::RichText::new("No models added").small());
    } else {
        let mut to_delete: Vec<usize> = Vec::new();
        let mut to_edit_idx: Option<usize> = None;
        for (i, m) in s.models.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(&m.name).strong());
                ui.label(egui::RichText::new(&m.description).small());
                if ui.button("Edit").clicked() {
                    to_edit_idx = Some(i);
                }
                if ui.button("Delete").clicked() {
                    to_delete.push(i);
                }
            });
            ui.add_space(5.0);
        }
        for &i in to_delete.iter().rev() {
            s.models.remove(i);
        }
        // If Edit was clicked and form is not yet open, open it now (avoids borrow conflicts).
        if let Some(idx) = to_edit_idx {
            if s.edit_model_form.is_none() {
                let m = s.models[idx].clone();
                s.edit_model_form = Some((idx, EditModelForm::from_model(&m)));
            }
        }
    }

    ui.add_space(10.0);
    if ui.button("+ Add Model").clicked() {
        s.add_model_form = Some(AddModelForm::new());
    }

    let mut form_cancelled = false;
    if let Some(mut form) = s.add_model_form.take() {
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);
        ui.label(egui::RichText::new("Add Model").strong());
        ui.horizontal(|ui| {
            ui.label("Name:");
            bounded_singleline(ui, &mut form.name);
        });
        ui.horizontal(|ui| {
            ui.label("Description:");
            let w = ui.available_width().max(120.0);
            ui.scope(|ui| {
                ui.set_max_width(w);
                ui.add(
                    egui::TextEdit::multiline(&mut form.description)
                        .desired_width(w)
                        .desired_rows(3),
                );
            });
        });
        ui.horizontal(|ui| {
            ui.label("Start Args:");
            let w = ui.available_width().max(120.0);
            ui.scope(|ui| {
                ui.set_max_width(w);
                ui.add(
                    egui::TextEdit::multiline(&mut form.args)
                        .desired_width(w)
                        .desired_rows(3)
                        .hint_text("e.g. --model path/to/model.gguf -c 2048 --ngl 99")
                        .frame(true),
                );
            });
        });
        ui.horizontal(|ui| {
            if ui.button("Add").clicked() {
                let nm = models::ModelConfig {
                    id: models::ModelConfig::new_unique_id(),
                    name: form.name.clone(),
                    description: form.description.clone(),
                    args: form.args.clone(),
                };
                s.models.push(nm);
                form_cancelled = true;
                // FIX: persist config to disk immediately after adding
                config.models = s.models.clone();
                let _ = config.save();
            }
            if ui.button("Cancel").clicked() {
                form_cancelled = true;
            }
        });
        if !form_cancelled {
            s.add_model_form = Some(form);
        }
    }

    let mut edit_cancelled = false;
    if let Some((edit_idx, mut form)) = s.edit_model_form.take() {
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);
        ui.label(egui::RichText::new("Edit Model").strong());
        ui.horizontal(|ui| {
            ui.label("Name:");
            bounded_singleline(ui, &mut form.name);
        });
        ui.horizontal(|ui| {
            ui.label("Description:");
            let w = ui.available_width().max(120.0);
            ui.scope(|ui| {
                ui.set_max_width(w);
                ui.add(
                    egui::TextEdit::multiline(&mut form.description)
                        .desired_width(w)
                        .desired_rows(3),
                );
            });
        });
        ui.horizontal(|ui| {
            ui.label("Start Args:");
            let w = ui.available_width().max(120.0);
            ui.scope(|ui| {
                ui.set_max_width(w);
                ui.add(
                    egui::TextEdit::multiline(&mut form.args)
                        .desired_width(w)
                        .desired_rows(3)
                        .hint_text("e.g. --model path/to/model.gguf -c 2048 --ngl 99")
                        .frame(true),
                );
            });
        });
        ui.horizontal(|ui| {
            if ui.button("Save Changes").clicked() {
                if let Some(model) = s.models.get_mut(edit_idx) {
                    model.name = form.name.clone();
                    model.description = form.description.clone();
                    model.args = form.args.clone();
                }
                edit_cancelled = true;
                config.models = s.models.clone();
                let _ = config.save();
            }
            if ui.button("Cancel").clicked() {
                edit_cancelled = true;
            }
        });
        if !edit_cancelled {
            s.edit_model_form = Some((edit_idx, form));
        }
    }

    ui.add_space(20.0);
    ui.separator();
    ui.add_space(10.0);
    if ui.button("Save").clicked() {
        // Persist all settings changes to disk
        config.llama_server_path = s.llama_server_path.clone();
        config.models = s.models.clone();
        if let Err(e) = config.save() {
            eprintln!("Error saving config: {}", e);
        }
    }
}

impl eframe::App for LlamaServerManagerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Sync in-memory config from settings
        self.config = self.settings.to_config();
        self.update_logs();
        if self.stop_requested {
            self.stop_current_model();
        }
        self.draw_home(ctx);
        self.draw_settings(ctx);
    }
}

fn main() -> eframe::Result {
    let opts = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Llama Server Manager"),
        ..Default::default()
    };
    eframe::run_native(
        "Llama Server Manager",
        opts,
        Box::new(|_cc| Ok(Box::new(LlamaServerManagerApp::new()))),
    )
}
