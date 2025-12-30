use crate::iconize::{IconManager, IconSelector, IconSource};
use crate::main_area::file_options::file_options;
use crate::main_area::left_controls::enums::SortOrder;
use eframe::egui::{self, Id, Popup, PopupCloseBehavior, RichText, Sense, Vec2};
use std::fs;
use std::path::Path;
use std::time::SystemTime;

pub struct FileTree {
    pub sort_order: SortOrder,
    pub rename: String,
    pub renaming_path: Option<String>,
    pub menu_error: String,
    pub new_folder_name: String,
    pub creating_folder_in: Option<String>,
}

impl Default for FileTree {
    fn default() -> Self {
        Self {
            sort_order: SortOrder::NameAZ,
            rename: String::new(),
            renaming_path: None,
            menu_error: String::new(),
            new_folder_name: String::new(),
            creating_folder_in: None,
        }
    }
}

impl FileTree {
    pub fn render(
        &mut self,
        ui: &mut egui::Ui,
        path: &str,
        current_file: &mut String,
        vault: &str,
        sort_entrys: &bool,
        enable_icons: bool,
        icon_manager: &mut IconManager,
        icon_selector: &mut IconSelector,
        depth: usize,
    ) {
        let indent_step = 12.0;
        let current_indent = depth as f32 * indent_step;

        let read_d = fs::read_dir(path);
        let entrys = match read_d {
            Ok(t) => t,
            Err(r) => {
                ui.label("Nothing to see here");
                ui.label(egui::RichText::new(r.to_string()).strong());
                return;
            }
        };

        let mut entrys_vec: Vec<String> = Vec::new();
        for entry in entrys {
            if let Ok(e) = entry {
                entrys_vec.push(e.path().to_str().unwrap().to_string());
            }
        }

        entrys_vec.sort_by(|a, b| {
            let path_a = Path::new(a);
            let path_b = Path::new(b);
            let get_modified = |p: &Path| {
                fs::metadata(p)
                    .and_then(|m| m.modified())
                    .unwrap_or(SystemTime::UNIX_EPOCH)
            };
            let get_created = |p: &Path| {
                fs::metadata(p)
                    .and_then(|m| m.created())
                    .unwrap_or(SystemTime::UNIX_EPOCH)
            };

            match self.sort_order {
                SortOrder::NameAZ => {
                    let a_is_dir = path_a.is_dir();
                    let b_is_dir = path_b.is_dir();
                    if a_is_dir && !b_is_dir {
                        std::cmp::Ordering::Less
                    } else if !a_is_dir && b_is_dir {
                        std::cmp::Ordering::Greater
                    } else {
                        path_a.file_name().cmp(&path_b.file_name())
                    }
                }
                SortOrder::NameZA => path_b.file_name().cmp(&path_a.file_name()),
                SortOrder::ModifiedNewOld => get_modified(path_b).cmp(&get_modified(path_a)),
                SortOrder::ModifiedOldNew => get_modified(path_a).cmp(&get_modified(path_b)),
                SortOrder::CreatedNewOld => get_created(path_b).cmp(&get_created(path_a)),
                SortOrder::CreatedOldNew => get_created(path_a).cmp(&get_created(path_b)),
            }
        });

        if self.creating_folder_in.as_deref() == Some(path) {
            ui.horizontal(|ui| {
                ui.add_space(current_indent + 34.0);
                let res = ui.text_edit_singleline(&mut self.new_folder_name);
                res.request_focus();

                if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    let new_path = Path::new(path).join(&self.new_folder_name);
                    if !new_path.exists() {
                        if fs::create_dir(&new_path).is_ok() {
                            self.creating_folder_in = None;
                            self.new_folder_name.clear();
                        } else {
                            self.menu_error = "Failed to create folder".to_string();
                        }
                    } else {
                        self.menu_error = "Folder already exists".to_string();
                    }
                } else if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.creating_folder_in = None;
                    self.new_folder_name.clear();
                }
            });
        }

        for file_location in entrys_vec {
            let path_obj = Path::new(&file_location);
            let is_dir = path_obj.is_dir();
            let file_name = path_obj
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            let relative_path = if let Ok(rel) = path_obj.strip_prefix(vault) {
                rel.to_string_lossy().replace('\\', "/")
            } else {
                file_name.to_string()
            };

            let mut icon_id: Option<String> = None;
            if enable_icons {
                if let Some(icon_str) = icon_manager.get_icon(&relative_path) {
                    icon_id = Some(icon_str.clone());
                }
            }

            let is_selected = &file_location == current_file;
            let row_size = Vec2::new(ui.available_width(), 18.0);

            if is_dir {
                let id = ui.make_persistent_id(&file_location);
                let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(
                    ui.ctx(),
                    id,
                    false,
                );

                let is_renaming = self
                    .renaming_path
                    .as_ref()
                    .map_or(false, |p| *p == file_location);

                if is_renaming {
                    ui.horizontal(|ui| {
                        ui.add_space(current_indent + 34.0);
                        let res = ui.text_edit_singleline(&mut self.rename);
                        res.request_focus();
                        if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            let new_path = path_obj.parent().unwrap().join(&self.rename);
                            if fs::rename(&file_location, &new_path).is_ok() {
                                // Update icons
                                let old_rel = Path::new(&file_location)
                                    .strip_prefix(vault)
                                    .map(|p| p.to_string_lossy().replace('\\', "/"))
                                    .unwrap_or_else(|_| file_location.clone());
                                let new_rel = new_path
                                    .strip_prefix(vault)
                                    .map(|p| p.to_string_lossy().replace('\\', "/"))
                                    .unwrap_or_else(|_| new_path.to_string_lossy().to_string());
                                icon_manager.rename_icon(vault, &old_rel, &new_rel);

                                if *current_file == file_location {
                                    *current_file = new_path.to_str().unwrap().to_string();
                                }
                                self.renaming_path = None;
                            } else {
                                self.menu_error = "Failed to rename folder".to_string();
                            }
                        } else if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                            self.renaming_path = None;
                        }
                    });
                } else {
                    let dnd_res = ui.dnd_drag_source(
                        Id::new("dnd_dir").with(&file_location),
                        file_location.clone(),
                        |ui| {
                            let (rect, response) =
                                ui.allocate_exact_size(row_size, Sense::click());

                            if response.hovered() || is_selected {
                                ui.painter().rect_filled(
                                    rect,
                                    2.0,
                                    if is_selected {
                                        ui.style().visuals.selection.bg_fill
                                    } else {
                                        ui.style().visuals.widgets.hovered.bg_fill
                                    },
                                );
                            }

                            // Flecha
                            let arrow_color = ui.visuals().widgets.noninteractive.fg_stroke.color;
                            let arrow_rect = egui::Rect::from_center_size(
                                rect.left_center() + egui::vec2(current_indent + 8.0, 0.0),
                                Vec2::new(10.0, 10.0),
                            );
                            let mut arrow_points = vec![
                                arrow_rect.left_top(),
                                arrow_rect.left_bottom(),
                                arrow_rect.right_center(),
                            ];
                            if state.is_open() {
                                let rotation =
                                    egui::emath::Rot2::from_angle(std::f32::consts::PI / 2.0);
                                for p in &mut arrow_points {
                                    *p = arrow_rect.center()
                                        + rotation * (*p - arrow_rect.center());
                                }
                            }
                            ui.painter().add(egui::Shape::convex_polygon(
                                arrow_points,
                                arrow_color,
                                egui::Stroke::NONE,
                            ));

                            // Icono
                            let icon_rect = egui::Rect::from_center_size(
                                rect.left_center() + egui::vec2(current_indent + 24.0, 0.0),
                                Vec2::new(14.0, 14.0),
                            );
                            if let Some(id) = &icon_id {
                                if let Some(source) = icon_manager.get_icon_source(id) {
                                    if let IconSource::Bytes(bytes) = source {
                                        ui.scope_builder(
                                            egui::UiBuilder::new().max_rect(icon_rect),
                                            |ui| {
                                                ui.add(
                                                    egui::Image::from_bytes(
                                                        format!("bytes://{}.svg", id),
                                                        bytes,
                                                    )
                                                    .fit_to_exact_size(Vec2::new(14.0, 14.0)),
                                                );
                                            },
                                        );
                                    }
                                } else {
                                    ui.scope_builder(
                                        egui::UiBuilder::new().max_rect(icon_rect),
                                        |ui| {
                                            egui_twemoji::EmojiLabel::new(id).show(ui);
                                        },
                                    );
                                }
                            }

                                                    // Nombre

                                                    let has_icon = icon_id.as_ref().map_or(false, |s| !s.is_empty());

                                                    let text_offset = if has_icon { 34.0 } else { 18.0 };

                                                    let text_pos = rect.left_center() + egui::vec2(current_indent + text_offset, 0.0);
                            ui.painter().text(
                                text_pos,
                                egui::Align2::LEFT_CENTER,
                                file_name,
                                egui::FontId::proportional(14.0),
                                ui.style().visuals.text_color(),
                            );

                            response
                        },
                    );

                    let response = dnd_res.response.interact(Sense::click());
                    if response.clicked() {
                        state.toggle(ui);
                    }
                    if response.double_clicked() {
                        self.renaming_path = Some(file_location.clone());
                        self.rename = file_name.to_string();
                    }

                    Popup::context_menu(&response)
                        .id(Id::new("ctx_dir").with(&file_location))
                        .show(|ui| {
                            if ui.button("New Folder").clicked() {
                                self.creating_folder_in = Some(file_location.clone());
                                self.new_folder_name = "New Folder".to_string();
                                ui.close();
                            }
                            ui.separator();
                            if enable_icons {
                                if ui.button("Change Icon").clicked() {
                                    icon_selector.open(relative_path.clone(), icon_manager);
                                    ui.close();
                                }
                                ui.separator();
                            }
                        });

                    if response.dnd_hover_payload::<String>().is_some() {
                        ui.painter().rect_stroke(
                            response.rect,
                            2.0,
                            egui::Stroke::new(
                                2.0,
                                ui.ctx().style().visuals.selection.stroke.color,
                            ),
                            egui::StrokeKind::Middle,
                        );
                    }
                    if let Some(source_path) = response.dnd_release_payload::<String>() {
                        let source_str: &str = &source_path;
                        if source_str != file_location && !file_location.starts_with(source_str) {
                            let target_path = Path::new(&file_location)
                                .join(Path::new(source_str).file_name().unwrap());
                            if let Err(e) = fs::rename(source_str, &target_path) {
                                self.menu_error = format!("Move error: {}", e);
                            } else {
                                // Update icons
                                let old_rel = Path::new(source_str)
                                    .strip_prefix(vault)
                                    .map(|p| p.to_string_lossy().replace('\\', "/"))
                                    .unwrap_or_else(|_| source_str.to_string());
                                let new_rel = target_path
                                    .strip_prefix(vault)
                                    .map(|p| p.to_string_lossy().replace('\\', "/"))
                                    .unwrap_or_else(|_| target_path.to_string_lossy().to_string());
                                icon_manager.rename_icon(vault, &old_rel, &new_rel);

                                if *current_file == source_str {
                                    *current_file = target_path.to_str().unwrap().to_string();
                                }
                            }
                        }
                    }
                }

                state.show_body_unindented(ui, |ui| {
                    self.render(
                        ui,
                        &file_location,
                        current_file,
                        vault,
                        sort_entrys,
                        enable_icons,
                        icon_manager,
                        icon_selector,
                        depth + 1,
                    );
                });
            } else {
                let is_renaming = self
                    .renaming_path
                    .as_ref()
                    .map_or(false, |p| *p == file_location);
                if is_renaming {
                    ui.horizontal(|ui| {
                        ui.add_space(current_indent + 34.0);
                        let res = ui.text_edit_singleline(&mut self.rename);
                        res.request_focus();
                        if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            let new_path = path_obj.parent().unwrap().join(&self.rename);
                            if fs::rename(&file_location, &new_path).is_ok() {
                                // Update icons
                                let old_rel = Path::new(&file_location).strip_prefix(vault).map(|p| p.to_string_lossy().replace('\\', "/")).unwrap_or_else(|_| file_location.clone());
                                let new_rel = new_path.strip_prefix(vault).map(|p| p.to_string_lossy().replace('\\', "/")).unwrap_or_else(|_| new_path.to_string_lossy().to_string());
                                icon_manager.rename_icon(vault, &old_rel, &new_rel);

                                if *current_file == file_location {
                                    *current_file = new_path.to_str().unwrap().to_string();
                                }
                                self.renaming_path = None;
                            }
                        } else if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                            self.renaming_path = None;
                        }
                    });
                } else {
                    let dnd_id = Id::new("dnd_file").with(&file_location);
                    let dnd_res = ui.dnd_drag_source(dnd_id, file_location.clone(), |ui| {
                        let (rect, response) = ui.allocate_exact_size(row_size, Sense::click());

                        if is_selected || response.hovered() {
                            ui.painter().rect_filled(
                                rect,
                                2.0,
                                if is_selected {
                                    ui.style().visuals.selection.bg_fill
                                } else {
                                    ui.style().visuals.widgets.hovered.bg_fill
                                },
                            );
                        }

                        // Icono (Personalizado o Espacio)
                        let icon_rect = egui::Rect::from_center_size(
                            rect.left_center() + egui::vec2(current_indent + 8.0, 0.0),
                            Vec2::new(14.0, 14.0),
                        );
                        if let Some(id) = &icon_id {
                            if let Some(source) = icon_manager.get_icon_source(id) {
                                if let IconSource::Bytes(bytes) = source {
                                        ui.scope_builder(egui::UiBuilder::new().max_rect(icon_rect), |ui| {
                                        ui.add(
                                            egui::Image::from_bytes(
                                                format!("bytes://{}.svg", id),
                                                bytes,
                                            )
                                            .fit_to_exact_size(Vec2::new(14.0, 14.0)),
                                        );
                                    });
                                }
                            } else {
                                // Emoji o texto usando egui-twemoji
                                    ui.scope_builder(egui::UiBuilder::new().max_rect(icon_rect), |ui| {
                                    egui_twemoji::EmojiLabel::new(id).show(ui);
                                });
                            }
                        }

                        let text_color = if is_selected {
                            ui.style().visuals.selection.stroke.color
                        } else {
                            ui.style().visuals.text_color()
                        };
                        let has_icon = icon_id.as_ref().map_or(false, |s| !s.is_empty());
                        let text_offset = if has_icon { 18.0 } else { 4.0 };
                        ui.painter().text(
                            rect.left_center() + egui::vec2(current_indent + text_offset, 0.0),
                            egui::Align2::LEFT_CENTER,
                            file_name,
                            egui::FontId::proportional(14.0),
                            text_color,
                        );

                        response
                    });

                    let response = dnd_res.response.interact(Sense::click());
                    if response.clicked() {
                        *current_file = file_location.clone();
                        ui.ctx().request_repaint();
                    }
                    if response.double_clicked() {
                        self.renaming_path = Some(file_location.clone());
                        self.rename = file_name.to_string();
                    }

                    Popup::context_menu(&response)
                        .id(Id::new("ctx").with(&file_location))
                        .show(|ui| {
                            if enable_icons {
                                if ui.button("Change Icon").clicked() {
                                    icon_selector.open(relative_path.clone(), icon_manager);
                                    ui.close();
                                }
                                ui.separator();
                            }
                            file_options(
                                ui,
                                &file_location,
                                &path,
                                &mut self.rename,
                                &mut self.renaming_path,
                                &mut self.menu_error,
                                vault,
                            );
                        });
                }
            }
            ui.add_space(2.0);
        }
    }
}
