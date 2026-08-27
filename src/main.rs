#![windows_subsystem = "windows"]

use std::process::Command;
use std::os::windows::process::CommandExt;

use eframe::egui;

mod config;
mod models;
mod ui;

/// Parse a shell-style arguments string into separate args, respecting quotes.
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
            }
            '"' => {
                if in_quotes {
                    in_quotes = false;
                    args.push(current.clone());
                    current.clear();
                } else if !current.is_empty() {
                    in_quotes = true;
                } else {
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

struct LlamaServerManagerApp {
    config: config::Config,
    settings: SettingsState,
    active_model_id: Option<String>,
    running_model: Option<models::RunningModel>,
    log_stream: Option<ui::logs::ModelLogStream>,
    model_logs: Vec<models::LogEntry>,
    auto_scroll: bool,
    open_settings: bool,
    stop_requested: bool,
    exit_dialog_shown: bool,
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
            exit_dialog_shown: false,
        }
    }

    fn show_shutdown_dialog(&mut self, ctx: &egui::Context) {
        let screen = ctx.screen_rect();
        let w = 380.0;
        let h = 160.0;
        let pos = egui::Pos2::new(
            screen.min.x + (screen.width() - w) / 2.0,
            screen.min.y + (screen.height() - h) / 2.0,
        );

        egui::Window::new("Confirm Exit")
            .collapsible(false)
            .resizable(false)
            .default_size([w, h])
            .current_pos(pos)
            .show(ctx, |ui| {
                ui.label("A model is currently running.");
                ui.add_space(5.0);
                ui.label("Stop the model and exit?");
                ui.add_space(15.0);
                ui.horizontal(|ui| {
                    if ui.button("Yes").clicked() {
                        self.stop_current_model();
                        // Let the app exit naturally after stopping the model.
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if ui.button("No").clicked() {
                        // Keep running — dismiss dialog and unblock closing (egui will ignore it).
                        self.exit_dialog_shown = false;
                        // Send CancelClose to be safe, though egui should handle it.
                        ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                    }
                });
            });
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
        let server_dir = server_path.parent().map(std::path::Path::to_path_buf);
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

    const MAX_LOG_LINES: usize = 2000;

    fn update_logs(&mut self, ctx: &egui::Context) {
        if let Some(ref mut log_stream) = self.log_stream {
            let new_lines = log_stream.poll(100);
            if !new_lines.is_empty() {
                let target_len = (self.model_logs.len() + new_lines.len()).min(Self::MAX_LOG_LINES);
                self.model_logs.reserve(new_lines.len());
                while self.model_logs.len() > target_len {
                    self.model_logs.remove(0);
                }
                self.model_logs.extend(new_lines);
            }
        }
        if self.running_model.is_some() {
            ctx.request_repaint();
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

    fn draw_home(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Llama Server Manager");
                ui.separator();
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Settings").clicked() {
                        self.open_settings = true;
                    }
                });
            });
        });

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
                        let display_lines: Vec<String> = self
                            .model_logs
                            .iter()
                            .take(200)
                            .map(|e| e.display_line())
                            .collect();
                        if display_lines.is_empty() {
                            ui.monospace("Waiting for model output...\n");
                        } else {
                            egui::ScrollArea::vertical()
                                .auto_shrink(false)
                                .stick_to_bottom(self.auto_scroll)
                                .show(ui, |ui| {
                                    ui.monospace(display_lines.join("\n"));
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
        egui::Window::new("Settings")
            .open(&mut self.open_settings)
            .resizable(false)
            .default_pos([50.0, 50.0])
            .default_size([520.0, 640.0])
            .max_size([530.0, 680.0])
            .show(ctx, |ui| {
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
        if let Some(idx) = to_edit_idx {
            if s.edit_model_form.is_none() {
                let m = s.models[idx].clone();
                s.edit_model_form = Some((idx, EditModelForm::from_model(&m)));
            }
        }
    }

    ui.add_space(10.0);
    if ui.button("+ Add Model").clicked() {
        // Close edit form if open — only one form at a time
        s.edit_model_form = None;
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
                let trimmed = form.name.trim().to_string();
                if !trimmed.is_empty() && !form.args.trim().is_empty() {
                    // Check for duplicate name.
                    if s.models.iter().any(|m| m.name == trimmed) {
                        eprintln!("Model with name '{}' already exists.", trimmed);
                    } else {
                        let id = format!("model_{}", rand::random::<u32>());
                        s.models.push(models::ModelConfig {
                            id: id.clone(),
                            name: trimmed,
                            description: form.description.trim().to_string(),
                            args: form.args.trim().to_string(),
                        });
                        // Persist immediately after adding.
                        config.models = s.models.clone();
                        let _ = config.save();
                    }
                } else if form.name.is_empty() || form.args.trim().is_empty() {
                    eprintln!("Name and Args cannot be empty.");
                } else {
                    eprintln!("Name is required and Args cannot be empty.");
                }
            }
            if ui.button("Cancel").clicked() {
                form_cancelled = true;
            }
        });
        if !form_cancelled {
            s.add_model_form = Some(form);
        }
    }

    let mut cancel_edit = false;
    if let Some((idx, mut edit_form)) = s.edit_model_form.take() {
        // Close add form if open — only one form at a time
        s.add_model_form = None;
        ui.add_space(10.0);
        ui.separator();
        ui.add_space(10.0);
        ui.label(egui::RichText::new("Edit Model").strong());
        ui.horizontal(|ui| {
            ui.label("Name:");
            bounded_singleline(ui, &mut edit_form.name);
        });
        ui.horizontal(|ui| {
            ui.label("Description:");
            let w = ui.available_width().max(120.0);
            ui.scope(|ui| {
                ui.set_max_width(w);
                ui.add(
                    egui::TextEdit::multiline(&mut edit_form.description)
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
                    egui::TextEdit::multiline(&mut edit_form.args)
                        .desired_width(w)
                        .desired_rows(3)
                        .hint_text("e.g. --model path/to/model.gguf -c 2048 --ngl 99")
                        .frame(true),
                );
            });
        });
        ui.horizontal(|ui| {
            if ui.button("Save").clicked() {
                let trimmed_name = edit_form.name.trim().to_string();
                let trimmed_args = edit_form.args.trim().to_string();
                if !trimmed_name.is_empty() && !trimmed_args.is_empty() {
                    // Check for duplicate name (ignoring the current entry).
                    let duplicate = s.models.iter().enumerate().any(|(i, m)| {
                        i != idx && m.name == trimmed_name
                    });
                    if duplicate {
                        eprintln!("Model with name '{}' already exists.", trimmed_name);
                    } else {
                        let new_config = models::ModelConfig {
                            id: s.models[idx].id.clone(),
                            name: trimmed_name,
                            description: edit_form.description.trim().to_string(),
                            args: trimmed_args,
                        };
                        // Model edited.
                        s.models[idx] = new_config;
                    }
                } else if edit_form.name.is_empty() || trimmed_args.is_empty() {
                    eprintln!("Name and Args cannot be empty.");
                } else {
                    eprintln!("Name is required and Args cannot be empty.");
                }
                // Persist immediately after editing.
                config.models = s.models.clone();
                let _ = config.save();
            }
            if ui.button("Cancel").clicked() {
                cancel_edit = true;
            }
        });
        if !cancel_edit {
            s.edit_model_form = Some((idx, edit_form));
        } else {
            // Cancelled — close the form.
            s.edit_model_form = None;
        }
    }

    // Final Save button to persist all settings (path + models).
    ui.add_space(20.0);
    ui.separator();
    ui.add_space(10.0);
    if ui.button("Save").clicked() {
        config.llama_server_path = s.llama_server_path.clone();
        config.models = s.models.clone();
        if let Err(e) = config.save() {
            eprintln!("Error saving config: {}", e);
        }
    }
}

impl eframe::App for LlamaServerManagerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update logs first so they're fresh when draw_home reads them.
        self.update_logs(ctx);

        self.config = self.settings.to_config();

        let close_requested = ctx.input(|i| i.viewport().close_requested());

        if close_requested && self.running_model.is_some() && !self.exit_dialog_shown {
            // Model is running — user clicked X. Show dialog and block closing.
            self.exit_dialog_shown = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }

        if self.stop_requested {
            self.stop_current_model();
        }

        // Always draw the main UI so the window keeps repainting.
        self.draw_home(ctx);
        self.draw_settings(ctx);

        // If showing dialog, draw it as an overlay on top of everything.
        if self.exit_dialog_shown {
            self.show_shutdown_dialog(ctx);
        }
    }
}

fn main() -> eframe::Result {
    let opts = eframe::NativeOptions {
        run_and_return: true, // Use on-demand event loop so update() is called for close_requested
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
