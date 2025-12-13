use eframe::egui;
use std::fs;
use std::path::Path;

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
        self.scan_dir(vault_path, vault_path);
        self.update_filter();
        self.initialized = true;
    }

    pub fn close(&mut self) {
        self.is_open = false;
    }

    fn scan_dir(&mut self, dir: &str, root: &str) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if !path.ends_with(".obsidian") && !path.ends_with(".git") {
                        self.scan_dir(path.to_str().unwrap(), root);
                    }
                } else {
                    if let Some(path_str) = path.to_str() {
                        self.all_files.push(path_str.to_string());
                    }
                }
            }
        }
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

    pub fn ui(&mut self, ctx: &egui::Context) -> Option<String> {
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
                }
            }
            if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                self.close();
            }

            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    //let results = self.filtered_results;
                    let mut salir = false;
                    for (i, path_str) in self.filtered_results.iter().enumerate() {
                        let file_name = Path::new(path_str).file_name().unwrap().to_str().unwrap();
                        let is_selected = i == self.selected_index;

                        let label = ui.selectable_label(is_selected, file_name);

                        if is_selected {
                            label.scroll_to_me(Some(egui::Align::Center));
                        }

                        if label.clicked() {
                            selected_file = Some(path_str.clone());
                            salir = true;
                        }
                    }

                    if self.filtered_results.is_empty() {
                        ui.weak("No matching files found.");
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
