use crate::iconize::{IconManager, IconSelector};
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
}

impl Default for FileTree {
    fn default() -> Self {
        Self {
            sort_order: SortOrder::NameAZ,
            rename: String::new(),
            renaming_path: None,
            menu_error: String::new(),
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
        sort_entrys: &bool, // Kept to match signature, but seemingly unused in original code snippet? No, it's used in arguments but logic inside?
        // Ah, in original render_files, `sort_entrys` was passed but I didn't see it used in the sorting logic block.
        // Wait, let's check the original code for `sort_entrys`.
        // It was passed to recursive calls.
        enable_icons: bool,
        icon_manager: &mut IconManager,
        icon_selector: &mut IconSelector,
    ) {
        let read_d = fs::read_dir(path);
        let entrys: fs::ReadDir;
        match read_d {
            Ok(t) => entrys = t,
            Err(r) => {
                ui.label("Nothing to see here");
                ui.label(egui::RichText::new(r.to_string()).strong());
                return;
            }
        }
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

        for file_location in entrys_vec {
            let file_name = Path::new(&file_location)
                .file_name()
                .expect("No fails")
                .to_str()
                .unwrap();

            // --- LÓGICA DE ICONOS ---
            let mut icon_data: Option<(bool, String)> = None; // (is_svg, content)
            let relative_path = if let Ok(rel) = Path::new(&file_location).strip_prefix(vault) {
                rel.to_str().unwrap_or(file_name).to_string()
            } else {
                file_name.to_string()
            };

            if enable_icons {
                if let Some(icon_str) = icon_manager.get_icon(&relative_path) {
                    // Verificar si es SVG
                    if let Some(svg_path) = icon_manager.get_icon_path(icon_str) {
                        icon_data = Some((true, svg_path.to_string()));
                    } else {
                        // Texto / Emoji
                        icon_data = Some((false, icon_str.clone()));
                    }
                }
            }

            if Path::new(&file_location).is_dir() {
                let id = ui.make_persistent_id(&file_location);
                let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(
                    ui.ctx(),
                    id,
                    false,
                );

                let (rect, response) = ui.allocate_exact_size(
                    Vec2::new(ui.available_width(), 18.0),
                    egui::Sense::click(),
                );

                if response.clicked() {
                    state.toggle(ui);
                }

                // Dibuja el fondo si está seleccionado (arrastrando sobre él)
                if response.dnd_hover_payload::<String>().is_some() {
                    ui.painter().rect_filled(
                        rect,
                        2.0,
                        ui.style().visuals.selection.bg_fill,
                    );
                }

                // Dibuja la flecha
                let arrow_color = ui.visuals().widgets.noninteractive.fg_stroke.color;
                let arrow_rect = egui::Rect::from_center_size(
                    rect.left_center() + egui::vec2(8.0, 0.0),
                    egui::Vec2::new(12.0, 12.0),
                );
                let mut arrow_points = vec![
                    arrow_rect.left_top(),
                    arrow_rect.right_top(),
                    arrow_rect.center_bottom(),
                ];
                if state.is_open() {
                    // Rotar la flecha
                    let rotation = egui::emath::Rot2::from_angle(std::f32::consts::PI / 2.0);
                    for p in &mut arrow_points {
                        *p = arrow_rect.center() + rotation * (*p - arrow_rect.center());
                    }
                }
                ui.painter()
                    .add(egui::Shape::convex_polygon(arrow_points, arrow_color, egui::Stroke::NONE));


                let mut icon_rect = egui::Rect::from_center_size(
                    rect.left_center() + egui::vec2(24.0, 0.0),
                    egui::Vec2::new(14.0, 14.0),
                );

                if let Some((is_svg, content)) = &icon_data {
                    if *is_svg {
                        let image_source = egui::ImageSource::Bytes {
                            uri: content.clone().into(),
                            bytes: std::fs::read(content).unwrap().into(),
                        };
                        let image = egui::Image::new(image_source)
                            .fit_to_exact_size(Vec2::new(14.0, 14.0));
                        ui.add(image);
                    } else {
                        ui.painter().text(
                            icon_rect.left_top(),
                            egui::Align2::LEFT_TOP,
                            content,
                            egui::FontId::proportional(14.0),
                            ui.style().visuals.text_color(),
                        );
                    }
                } else {
                    // Si no hay icono, desplaza el texto a la izquierda
                    icon_rect.set_width(0.0);
                }

                // Dibuja el nombre del archivo
                let text_rect = egui::Rect::from_min_max(
                    egui::pos2(icon_rect.right() + 4.0, rect.top()),
                    rect.right_bottom(),
                );
                ui.painter().text(
                    text_rect.left_top(),
                    egui::Align2::LEFT_TOP,
                    file_name,
                    egui::FontId::proportional(14.0),
                    ui.style().visuals.text_color(),
                );

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
                    );
                });

                if response.dnd_hover_payload::<String>().is_some() {
                    ui.painter().rect_stroke(
                        rect,
                        2.0,
                        egui::Stroke::new(2.0, ui.ctx().style().visuals.selection.stroke.color),
                        egui::StrokeKind::Middle,
                    );
                }

                if let Some(source_path) = response.dnd_release_payload::<String>() {
                    let source_str: &str = &source_path;
                    if source_str != file_location && !file_location.starts_with(source_str) {
                        let source_path_obj = Path::new(source_str);
                        let file_name_only = source_path_obj.file_name().unwrap();
                        let target_path = Path::new(&file_location).join(file_name_only);

                        if let Err(e) = fs::rename(source_str, &target_path) {
                            self.menu_error = format!("Move error: {}", e);
                        } else {
                            if *current_file == source_str {
                                *current_file = target_path.to_str().unwrap().to_string();
                            }
                        }
                    }
                }
            } else {
                let is_renaming_this = self
                    .renaming_path
                    .as_ref()
                    .map_or(false, |p| *p == file_location);

                if is_renaming_this {
                    let response = ui.text_edit_singleline(&mut self.rename);
                    response.request_focus();

                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        let path_obj = Path::new(&file_location);
                        let parent = path_obj.parent().unwrap();
                        let new_path = parent.join(&self.rename);

                        match fs::rename(&file_location, &new_path) {
                            Ok(_) => {
                                if *current_file == file_location {
                                    *current_file = new_path.to_str().unwrap().to_string();
                                }
                                self.renaming_path = None;
                            }
                            Err(e) => {
                                self.menu_error = format!("Error renaming: {}", e);
                            }
                        }
                    } else if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        self.renaming_path = None;
                    }
                } else {
                    let is_selected = &file_location == current_file;

                    let item_id = Id::new("dnd_file").with(&file_location);
                    let payload = file_location.clone();

                    let dnd_response = ui.dnd_drag_source(item_id, payload, |ui| {
                        // 1. Asignamos espacio para toda la fila
                        let row_height = 18.0; // Altura cómoda para texto e icono
                        let (rect, response) = ui.allocate_exact_size(
                            Vec2::new(ui.available_width(), row_height),
                            Sense::click(),
                        );

                        // 2. Pintamos el fondo si está seleccionado o hover
                        if ui.is_rect_visible(rect) {
                            if is_selected {
                                ui.painter().rect_filled(
                                    rect,
                                    2.0, // Redondeo ligero
                                    ui.style().visuals.selection.bg_fill,
                                );
                            } else if response.hovered() {
                                ui.painter().rect_filled(
                                    rect,
                                    2.0,
                                    ui.style().visuals.widgets.hovered.bg_fill,
                                );
                            }

                            // 3. Renderizamos el contenido dentro del rect
                            // CORREGIDO: Usamos allocate_ui_at_rect en lugar de child_ui
                            ui.allocate_ui_at_rect(rect, |ui| {
                                ui.horizontal(|ui| {
                                    ui.set_height(row_height); // Asegurar altura
                                    ui.add_space(4.0); // Margen izquierdo

                                    // Render Icono
                                    if let Some((is_svg, content)) = &icon_data {
                                        if *is_svg {
                                            let img_color = if is_selected {
                                                egui::Color32::WHITE
                                            } else {
                                                ui.style().visuals.text_color()
                                            };

                                            ui.add(
                                                egui::Image::new(&*content)
                                                    .tint(img_color)
                                                    .fit_to_exact_size(Vec2::new(14.0, 14.0)),
                                            );
                                        } else {
                                            ui.label(content); // Emoji
                                        }
                                        ui.add_space(4.0);
                                    }

                                    // Render Texto
                                    let text_color = if is_selected {
                                        ui.style().visuals.selection.stroke.color
                                    } else {
                                        ui.style().visuals.text_color()
                                    };

                                    ui.label(RichText::new(file_name).color(text_color));
                                });
                            });
                        }
                    });

                    let response = dnd_response.response.interact(egui::Sense::click());

                    let popup_id = Id::new("file_menu").with(&file_location);
                    Popup::context_menu(&response)
                        .id(popup_id)
                        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
                        .show(|ui| {
                            // Opción Change Icon (solo si están habilitados)
                            if enable_icons {
                                if ui.button("Change Icon").clicked() {
                                    icon_selector.open(relative_path.clone(), icon_manager);
                                    ui.close(); // CORREGIDO: close_menu -> close
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

                    if response.double_clicked() {
                        self.renaming_path = Some(file_location.clone());
                        self.rename = file_name.to_string();
                    } else if response.clicked() {
                        *current_file = file_location.to_string();
                    }
                }
            }
            ui.add_space(2.0);
        }
    }
}
