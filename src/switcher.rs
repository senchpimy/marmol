use eframe::egui;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub struct QuickSwitcher {
    pub is_open: bool,
    query: String,
    all_files: Vec<String>,
    filtered_results: Vec<String>,
    selected_index: usize,
    initialized: bool,
}

impl Default for QuickSwitcher {
    fn default() -> Self {
        Self {
            is_open: false,
            query: String::new(),
            all_files: Vec::new(),
            filtered_results: Vec::new(),
            selected_index: 0,
            initialized: false,
        }
    }
}

impl QuickSwitcher {
    pub fn open(&mut self, vault_path: &str) {
        self.is_open = true;
        self.query.clear();
        self.selected_index = 0;

        self.all_files.clear();
        self.scan_dir(vault_path);
        self.update_filter();
        self.initialized = true;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    fn scan_dir(&mut self, vault_path: &str) {
        use walkdir::WalkDir;
        self.all_files = WalkDir::new(vault_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                let path = e.path();
                if path.is_dir() {
                    return false;
                }
                // Skip .obsidian and .git folders
                !path.components().any(|c| {
                    let s = c.as_os_str().to_str().unwrap_or("");
                    s == ".obsidian" || s == ".git"
                })
            })
            .filter_map(|e| e.path().to_str().map(|s| s.to_string()))
            .collect();
    }

    fn update_filter(&mut self) {
        if self.query.is_empty() {
            self.filtered_results = self.all_files.clone();
        } else {
            let q = self.query.to_lowercase();
            self.filtered_results = self
                .all_files
                .iter()
                .filter(|path| {
                    let name = Path::new(path)
                        .file_name()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or("")
                        .to_lowercase();
                    name.contains(&q)
                })
                .cloned()
                .collect();
        }
        if self.selected_index >= self.filtered_results.len() {
            self.selected_index = 0;
        }
    }

    pub fn ui(&mut self, ctx: &egui::Context, vault_path: &str) -> Option<String> {
        let mut selected_file = None;

        if !self.is_open {
            return None;
        }

        let modal = egui::Window::new("Quick Switcher")
            .anchor(egui::Align2::CENTER_TOP, [0.0, 100.0])
            .fixed_size([500.0, 300.0])
            .title_bar(false)
            .collapsible(false)
            .resizable(false);

        modal.show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(egui::RichText::new("Open File").strong());
            });
            ui.add_space(5.0);

            let response = ui.add(
                egui::TextEdit::singleline(&mut self.query)
                    .hint_text("Type to search...")
                    .lock_focus(true),
            );

            if self.initialized {
                response.request_focus();
                self.initialized = false;
            }

            if response.changed() {
                self.update_filter();
                self.selected_index = 0;
            }

            if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                if self.selected_index + 1 < self.filtered_results.len() {
                    self.selected_index += 1;
                }
            }
            if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                if !self.filtered_results.is_empty() {
                    selected_file = Some(self.filtered_results[self.selected_index].clone());
                    self.close();
                } else {
                    let input_filename = self.query.clone();
                    if !input_filename.trim().is_empty() {
                        let full_path = PathBuf::from(vault_path).join(&input_filename);
                        let new_file_path = full_path.with_extension("md");

                        if let Some(parent) = new_file_path.parent() {
                            if !parent.exists() {
                                if let Err(e) = fs::create_dir_all(parent) {
                                    eprintln!("Failed to create directories for new file: {}", e);
                                }
                            }
                        }

                        match fs::File::create(&new_file_path) {
                            Ok(mut file) => {
                                let default_content = "";
                                if let Err(e) = file.write_all(default_content.as_bytes()) {
                                    eprintln!("Failed to write to new file: {}", e);
                                } else {
                                    selected_file =
                                        Some(new_file_path.to_string_lossy().to_string());
                                    self.close();
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to create new file: {}", e);
                            }
                        }
                    }
                }
            }
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.close();
            }

            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(250.0)
                .show(ui, |ui| {
                    let mut salir = false;
                    for (i, path_str) in self.filtered_results.iter().take(100).enumerate() {
                        let rel_path = path_str.strip_prefix(vault_path).unwrap_or(path_str);
                        let rel_path = rel_path.strip_prefix('/').unwrap_or(rel_path);
                        
                        let path = Path::new(rel_path);
                        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        let parent_dir = path.parent().and_then(|p| p.to_str()).unwrap_or("");

                        let is_selected = i == self.selected_index;

                        let mut job = egui::text::LayoutJob::default();
                        let color = if is_selected {
                            ui.visuals().strong_text_color()
                        } else {
                            ui.visuals().text_color()
                        };

                        let regular_format = egui::TextFormat {
                            color,
                            font_id: egui::FontId::proportional(14.0),
                            ..Default::default()
                        };
                        let bold_format = egui::TextFormat {
                            color: ui.visuals().strong_text_color(),
                            font_id: egui::FontId::proportional(14.0),
                            ..Default::default()
                        };
                        let weak_format = egui::TextFormat {
                            color: ui.visuals().weak_text_color(),
                            font_id: egui::FontId::proportional(12.0),
                            ..Default::default()
                        };

                        // Add parent directory if it exists
                        if !parent_dir.is_empty() {
                            job.append(&format!("{}/", parent_dir), 0.0, weak_format);
                        }

                        if self.query.is_empty() {
                            job.append(file_name, 0.0, regular_format.clone());
                        } else {
                            let q = self.query.to_lowercase();
                            let name_lower = file_name.to_lowercase();
                            if let Some(start) = name_lower.find(&q) {
                                let end = start + q.len();
                                job.append(&file_name[..start], 0.0, regular_format.clone());
                                job.append(&file_name[start..end], 0.0, bold_format);
                                job.append(&file_name[end..], 0.0, regular_format.clone());
                            } else {
                                job.append(file_name, 0.0, regular_format.clone());
                            }
                        }

                        let label = ui.selectable_label(is_selected, job);

                        if is_selected {
                            label.scroll_to_me(Some(egui::Align::Center));
                        }

                        if label.clicked() {
                            selected_file = Some(path_str.clone());
                            salir = true;
                        }
                    }

                    if self.filtered_results.is_empty() {
                        if !self.query.trim().is_empty() {
                            let new_file_name = format!("{}.md", self.query);
                            ui.label(format!("Press Enter to create new file: {}", new_file_name));
                        } else {
                            ui.weak("No matching files found.");
                        }
                    }
                    if salir {
                        self.close();
                    }
                });
        });

        ctx.move_to_top(egui::LayerId::new(
            egui::Order::Foreground,
            egui::Id::new("Quick Switcher"),
        ));

        selected_file
    }
}
