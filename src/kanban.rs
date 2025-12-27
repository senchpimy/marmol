use egui::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use regex::Regex;
use walkdir::WalkDir;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KanbanTask {
    pub content: String,
    pub completed: bool,
    pub date: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KanbanColumn {
    pub title: String,
    pub tasks: Vec<KanbanTask>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct KanbanBoard {
    pub columns: Vec<KanbanColumn>,
    pub archive: Vec<KanbanTask>,
    pub settings: String,
    pub frontmatter: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Location {
    pub col: usize,
    pub row: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColumnLocation {
    pub col: usize,
}

#[derive(Default)]
pub struct KanbanGui {
    pub board: KanbanBoard,
    pub path: String,
    pub new_task_str: String,
    pub adding_task_to: Option<usize>,
    pub editing_task: Option<(usize, usize)>,
}

impl KanbanGui {
    pub fn set_path(&mut self, path: &str) {
        if self.path != path {
            self.path = path.to_string();
            self.load();
        }
    }

    pub fn load(&mut self) {
        if let Ok(content) = fs::read_to_string(&self.path) {
            self.board = parse_kanban(&content);
        }
    }

    pub fn save(&self) {
        if self.path.is_empty() {
            return;
        }
        let content = serialize_kanban(&self.board);
        if let Ok(mut file) = fs::File::create(&self.path) {
            let _ = file.write_all(content.as_bytes());
        }
    }

    pub fn show(&mut self, ui: &mut Ui, vault: &str) -> Option<String> {
        let mut open_file = None;
        let mut needs_save = false;
        
        // Variables para el movimiento (Source -> Destination)
        let mut from_col: Option<usize> = None;
        let mut to_col: Option<usize> = None;
        
        let mut from_task: Option<Location> = None;
        let mut to_task: Option<Location> = None;
        
        let mut adding_task_to = self.adding_task_to;
        let mut editing_task = self.editing_task;
        let mut new_task_str = self.new_task_str.clone();
        let mut remove_task = None;
        let mut archive_task = None;

        let re_link = Regex::new(r"\[\[(.*?)\]\]").unwrap();

        if ui.input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape)) {
            adding_task_to = None;
            editing_task = None;
        }

        ui.add_space(10.0);
        
        ScrollArea::both().auto_shrink([false, true]).show(ui, |ui| {
            ui.horizontal_top(|ui| {
                // Iteramos sobre las columnas
                for col_idx in 0..self.board.columns.len() {
                    
                    // Contenedor visual de la columna
                    ui.vertical(|ui| {
                        ui.set_width(260.0);
                        
                        let col_frame = Frame::group(ui.style())
                            .fill(ui.visuals().faint_bg_color)
                            .inner_margin(8.0)
                            .corner_radius(8.0);

                        let (_, dropped_task_payload) = ui.dnd_drop_zone::<Location, ()>(col_frame, |ui| {
                            ui.set_min_size(vec2(250.0, 100.0));
                            
                            ui.horizontal(|ui| {
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if ui.button("➕").clicked() {
                                        adding_task_to = Some(col_idx);
                                        new_task_str = String::new();
                                    }
                                    
                                    let header_id = Id::new("col_header").with(col_idx);
                                    let header_res = ui.dnd_drag_source(header_id, ColumnLocation { col: col_idx }, |ui| {
                                        ui.set_width(ui.available_width());
                                        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                                            ui.heading(&self.board.columns[col_idx].title);
                                        });
                                    });

                                    let response = header_res.response;
                                    if let (Some(pointer), Some(_payload)) = (
                                        ui.input(|i| i.pointer.interact_pos()),
                                        response.dnd_hover_payload::<ColumnLocation>(),
                                    ) {
                                        let rect = response.rect;
                                        let stroke = Stroke::new(3.0, ui.visuals().selection.bg_fill);
                                        
                                        let insert_col_idx = if pointer.x < rect.center().x {
                                            ui.painter().vline(rect.left(), rect.y_range(), stroke);
                                            col_idx
                                        } else {
                                            ui.painter().vline(rect.right(), rect.y_range(), stroke);
                                            col_idx + 1
                                        };

                                        if let Some(dragged_payload) = response.dnd_release_payload::<ColumnLocation>() {
                                            from_col = Some(dragged_payload.col);
                                            to_col = Some(insert_col_idx);
                                        }
                                    }
                                });
                            });

                            ui.add_space(4.0);
                            ui.separator();
                            ui.add_space(4.0);

                            ui.vertical(|ui| {
                                ui.set_min_height(200.0);
                                
                                // Input nueva tarea
                                if adding_task_to == Some(col_idx) {
                                    ui.horizontal(|ui| {
                                        let res = ui.add(TextEdit::singleline(&mut new_task_str).hint_text("Nueva tarea..."));
                                        if res.lost_focus() || ui.input(|i| i.key_pressed(Key::Enter)) {
                                        }
                                    });
                                    ui.add_space(4.0);
                                }

                                for task_idx in 0..self.board.columns[col_idx].tasks.len() {
                                    let item_id = Id::new("kanban_item").with(col_idx).with(task_idx);
                                    let item_location = Location { col: col_idx, row: task_idx };
                                    
                                    let dnd_res = ui.dnd_drag_source(item_id, item_location, |ui| {
                                        Frame::NONE
                                            .fill(ui.visuals().window_fill())
                                            .inner_margin(8.0)
                                            .corner_radius(6.0)
                                            .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                                            .show(ui, |ui| {
                                                ui.set_width(ui.available_width());
                                                ui.vertical(|ui| {
                                                    let task = &mut self.board.columns[col_idx].tasks[task_idx];
                                                    ui.horizontal(|ui| {
                                                        let check_res = ui.checkbox(&mut task.completed, "");
                                                        if check_res.changed() { needs_save = true; }
                                                        
                                                        if editing_task == Some((col_idx, task_idx)) {
                                                            let res = ui.text_edit_singleline(&mut task.content);
                                                            if res.lost_focus() || ui.input(|i| i.key_pressed(Key::Enter)) {
                                                                editing_task = None;
                                                                needs_save = true;
                                                            }
                                                        } else {
                                                            ui.horizontal_wrapped(|ui| {
                                                                ui.spacing_mut().item_spacing.x = 0.0;
                                                                let mut last_end = 0;
                                                                for cap in re_link.captures_iter(&task.content) {
                                                                    let m = cap.get(0).unwrap();
                                                                    let start = m.start();
                                                                    let end = m.end();
                                                                    let link_text = cap.get(1).unwrap().as_str();
                                                                    if start > last_end { ui.label(&task.content[last_end..start]); }
                                                                    let accent = ui.visuals().selection.bg_fill;
                                                                    let link_label = Label::new(RichText::new(link_text).color(accent)).sense(Sense::click());
                                                                    if ui.add(link_label).clicked() {
                                                                        if let Some(path) = find_file(vault, link_text) {
                                                                            open_file = Some(path);
                                                                        }
                                                                    }
                                                                    last_end = end;
                                                                }
                                                                if last_end < task.content.len() { ui.label(&task.content[last_end..]); }
                                                            });
                                                        }
                                                    });
                                                    if let Some(date) = &task.date {
                                                        ui.weak(format!("📅 {}", date));
                                                    }
                                                });
                                            });
                                    });
                                    
                                    let response = dnd_res.response;
                                    
                                    response.context_menu(|ui| {
                                        if ui.button("📁 Archivar").clicked() { archive_task = Some((col_idx, task_idx)); ui.close(); }
                                        if ui.button("🗑 Eliminar").clicked() { remove_task = Some((col_idx, task_idx)); ui.close(); }
                                    });

                                    if response.double_clicked() { editing_task = Some((col_idx, task_idx)); }

                                    if let (Some(pointer), Some(_payload)) = (
                                        ui.input(|i| i.pointer.interact_pos()),
                                        response.dnd_hover_payload::<Location>(),
                                    ) {
                                        let rect = response.rect;
                                        let stroke = Stroke::new(2.0, ui.visuals().selection.bg_fill);
                                        
                                        let insert_row_idx = if pointer.y < rect.center().y {
                                            ui.painter().hline(rect.x_range(), rect.top(), stroke);
                                            task_idx
                                        } else {
                                            ui.painter().hline(rect.x_range(), rect.bottom(), stroke);
                                            task_idx + 1
                                        };

                                        if let Some(dragged_payload) = response.dnd_release_payload::<Location>() {
                                            from_task = Some(*dragged_payload);
                                            to_task = Some(Location { col: col_idx, row: insert_row_idx });
                                        }
                                    }
                                    ui.add_space(4.0);
                                }
                            });
                        });

                        if let Some(payload) = dropped_task_payload {
                            if to_task.is_none() {
                                from_task = Some(*payload);
                                to_task = Some(Location { col: col_idx, row: usize::MAX });
                            }
                        }
                    });
                    
                    ui.add_space(10.0);
                }
            });
        });

        if let (Some(f), Some(mut t)) = (from_col, to_col) {
            if f < self.board.columns.len() {
                if f < t { t = t.saturating_sub(1); }
                
                if f != t {
                    let col = self.board.columns.remove(f);
                    let target = t.min(self.board.columns.len());
                    self.board.columns.insert(target, col);
                    needs_save = true;
                }
            }
        }

        if let (Some(f), Some(mut t)) = (from_task, to_task) {
            if f.col < self.board.columns.len() && f.row < self.board.columns[f.col].tasks.len() && t.col < self.board.columns.len() {
                if f.col == t.col && f.row < t.row { 
                    t.row = t.row.saturating_sub(1); 
                }
                
                if f.col != t.col || f.row != t.row {
                    let task = self.board.columns[f.col].tasks.remove(f.row);
                    let target_row = t.row.min(self.board.columns[t.col].tasks.len());
                    self.board.columns[t.col].tasks.insert(target_row, task);
                    needs_save = true;
                }
            }
        }

        if let Some((c, t)) = remove_task {
            if c < self.board.columns.len() && t < self.board.columns[c].tasks.len() {
                self.board.columns[c].tasks.remove(t);
                needs_save = true;
            }
        }

        if let Some((c, t)) = archive_task {
            if c < self.board.columns.len() && t < self.board.columns[c].tasks.len() {
                let task = self.board.columns[c].tasks.remove(t);
                self.board.archive.push(task);
                needs_save = true;
            }
        }

        if let Some(c_idx) = adding_task_to {
            if self.adding_task_to == Some(c_idx) && ui.input(|i| i.key_pressed(Key::Enter)) && !new_task_str.is_empty() {
                if c_idx < self.board.columns.len() {
                    self.board.columns[c_idx].tasks.push(KanbanTask {
                        content: new_task_str.clone(),
                        completed: false,
                        date: None,
                    });
                    new_task_str = String::new();
                    adding_task_to = None;
                    needs_save = true;
                }
            }
        }

        self.adding_task_to = adding_task_to;
        self.editing_task = editing_task;
        self.new_task_str = new_task_str;

        if needs_save {
            self.save();
        }
        open_file
    }
}

fn parse_kanban(content: &str) -> KanbanBoard {
    let mut board = KanbanBoard::default();
    let mut current_column: Option<KanbanColumn> = None;
    let mut in_frontmatter = false;
    let mut frontmatter = String::new();
    let mut settings = String::new();
    let mut in_settings = false;
    let mut in_archive = false;

    for line in content.lines() {
        if line.trim() == "---" {
            if !in_frontmatter && frontmatter.is_empty() {
                in_frontmatter = true;
                continue;
            } else if in_frontmatter {
                in_frontmatter = false;
                continue;
            }
        }

        if in_frontmatter {
            frontmatter.push_str(line);
            frontmatter.push('\n');
            continue;
        }

        if line.starts_with("%% kanban:settings") {
            in_settings = true;
            continue;
        }
        if in_settings {
            if line.starts_with("%% ") {
                in_settings = false;
            } else {
                settings.push_str(line);
                settings.push('\n');
            }
            continue;
        }

        if line.starts_with("## Archive") {
            in_archive = true;
            if let Some(col) = current_column.take() {
                board.columns.push(col);
            }
            continue;
        }

        if line.starts_with("## ") {
            in_archive = false;
            if let Some(col) = current_column.take() {
                board.columns.push(col);
            }
            current_column = Some(KanbanColumn {
                title: line[3..].to_string(),
                tasks: Vec::new(),
            });
        } else if line.trim().starts_with("- [") {
            let completed = line.contains("- [x]");
            let rest = &line[6..];
            
            let mut content_str = rest.to_string();
            let mut date = None;
            
            if let Some(idx) = content_str.find("@{ ") {
                if let Some(end_idx) = content_str[idx..].find('}') {
                    date = Some(content_str[idx+2..idx+end_idx].to_string());
                    content_str = content_str[..idx].trim().to_string();
                }
            }

            let task = KanbanTask {
                content: content_str,
                completed,
                date,
            };

            if in_archive {
                board.archive.push(task);
            } else if let Some(ref mut col) = current_column {
                col.tasks.push(task);
            }
        }
    }

    if let Some(col) = current_column {
        board.columns.push(col);
    }

    board.frontmatter = frontmatter;
    board.settings = settings;
    board
}

fn serialize_kanban(board: &KanbanBoard) -> String {
    let mut out = String::new();
    out.push_str("---\n");
    out.push_str(&board.frontmatter);
    if !board.frontmatter.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("---\n\n");

    for col in &board.columns {
        out.push_str(&format!("## {}\n\n", col.title));
        for task in &col.tasks {
            let check = if task.completed { "x" } else { " " };
            let date_str = task.date.as_ref().map_or(String::new(), |d| format!(" @{{{}}}", d));
            out.push_str(&format!("- [{}] {}{}\n", check, task.content, date_str));
        }
        out.push_str("\n\n");
    }

    if !board.archive.is_empty() {
        out.push_str("## Archive\n\n");
        for task in &board.archive {
            let check = if task.completed { "x" } else { " " };
            let date_str = task.date.as_ref().map_or(String::new(), |d| format!(" @{{{}}}", d));
            out.push_str(&format!("- [{}] {}{}\n", check, task.content, date_str));
        }
        out.push_str("\n\n");
    }

    if !board.settings.is_empty() {
        out.push_str("%% kanban:settings\n");
        out.push_str(&board.settings);
        out.push_str("%%\n");
    }
    out
}

fn find_file(vault: &str, name: &str) -> Option<String> {
    let target_name = name.to_string();
    let target_name_md = format!("{}.md", name);

    for entry in WalkDir::new(vault).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Some(fname) = entry.file_name().to_str() {
                 if fname == target_name || fname == target_name_md {
                     return Some(entry.path().to_str()?.to_string());
                 }
            }
        }
    }
    None
}
