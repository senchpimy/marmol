use chrono::Local;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;

use egui::*;
use egui_plot::{GridMark, Line, Plot, Points};
use std::ops::RangeInclusive;

// --- Estructuras de Datos ---

#[derive(Debug)]
struct EditDay {
    index: i32,
    edit: Edit,
}

#[derive(Debug, PartialEq)]
enum Edit {
    Tittle,
    Description,
    Null,
}

#[derive(Serialize, Deserialize, Debug)]
struct TaskToDo {
    id: u32,
    name: String,
    description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TasksToDo {
    contents: Vec<TaskToDo>,
}

#[derive(Serialize, Deserialize, Debug)]
struct TaskCompleted {
    id: u32,
    completed: bool,
}
impl TaskCompleted {
    fn new(id: u32) -> Self {
        Self {
            id,
            completed: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Day {
    date: String,
    notes: Option<String>,
    tasks: Vec<TaskCompleted>,
}

impl Day {
    fn new(date: &str, tasks: Vec<TaskCompleted>) -> Self {
        Self {
            date: String::from(date),
            notes: None,
            tasks,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TasksFile {
    tasks: Vec<TaskToDo>,
    days: Vec<Day>,
    top_id: u32,
}

impl Default for TasksFile {
    fn default() -> Self {
        TasksFile {
            tasks: Vec::new(),
            days: Vec::new(),
            top_id: 0,
        }
    }
}

pub fn load_tasks(path: &str) -> TasksFile {
    let data = match fs::read_to_string(Path::new(path)) {
        Ok(x) => x,
        Err(_) => String::from("{\"tasks\":[],\"days\":[],\"top_id\":0}"),
    };
    match serde_json::from_str(&data) {
        Ok(t) => t,
        Err(_) => TasksFile::default(),
    }
}

// --- GUI Struct y Lógica ---

pub struct TasksGui {
    json_content: TasksFile,
    path: String,

    // Datos para la gráfica
    numeros_grafica: Vec<u16>,
    labels_grafica: Vec<String>, // Para mostrar fechas en eje X
    prom: f32,

    // Estados de UI
    add_task: bool,
    new_task: String,
    new_task_desc: String,
    edit: String,
    edit_task: EditDay,
    new_task_update: bool,
    add_entry: bool,
    new_entry_tittle: String,
    tasks_hash: HashMap<u32, String>,
    update_graph: bool,
    save_file: bool,
}

impl Default for TasksGui {
    fn default() -> Self {
        Self {
            save_file: false,
            add_entry: false,
            update_graph: false,
            json_content: TasksFile::default(),
            path: String::new(),
            numeros_grafica: vec![],
            labels_grafica: vec![],
            add_task: false,
            new_task: String::new(),
            edit_task: EditDay {
                index: -1,
                edit: Edit::Null,
            },
            edit: String::new(),
            new_entry_tittle: String::new(),
            new_task_desc: String::new(),
            new_task_update: false,
            tasks_hash: HashMap::new(),
            prom: 0.0,
        }
    }
}

impl TasksGui {
    pub fn set_tasks(&mut self, json_content: TasksFile) {
        self.json_content = json_content;
        self.rebuild_hashmap();
    }

    fn rebuild_hashmap(&mut self) {
        self.tasks_hash.clear();
        for element in &self.json_content.tasks {
            self.tasks_hash.insert(element.id, element.name.clone());
        }
    }

    pub fn set_path(&mut self, path: &str) {
        if path != &self.path {
            self.path = String::from(path);
            self.set_tasks(load_tasks(&self.path));
            self.calculate_stats();
        }
    }

    // Lógica separada para recalcular la gráfica y stats
    fn calculate_stats(&mut self) {
        let mut prom: u16 = 0;
        let mut tot_num = 0.0;

        self.numeros_grafica.clear();
        self.labels_grafica.clear();

        for element in &self.json_content.days {
            let mut tot: u16 = 0;
            element
                .tasks
                .iter()
                .for_each(|val| tot += val.completed as u16);
            prom += tot;

            self.numeros_grafica.push(tot);
            self.labels_grafica.push(element.date.clone());
            tot_num += 1.0;
        }

        // Invertimos porque 'days' suele tener lo más reciente al inicio (index 0),
        // pero la gráfica se dibuja de izquierda (viejo) a derecha (nuevo).
        self.numeros_grafica.reverse();
        self.labels_grafica.reverse();

        if tot_num > 0.0 {
            self.prom = prom as f32 / tot_num;
        } else {
            self.prom = 0.0;
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        // --- 1. Gráfica ---
        let labels = &self.labels_grafica;

        let x_fmt = move |x: GridMark, _range: &RangeInclusive<f64>| {
            let i = x.value as usize;
            if i < labels.len() {
                labels[i].clone()
            } else {
                String::new()
            }
        };

        ui.label(RichText::new(format!("Average: {:.1} tasks/day", self.prom)).strong());

        let markers_plot = Plot::new("Graph")
            .height(200.0)
            .x_axis_formatter(x_fmt)
            .data_aspect(0.5)
            .auto_bounds(true)
            .clamp_grid(true);

        markers_plot.show(ui, |plot_ui| {
            let mut num = 0.0;
            let mut line_points = vec![];
            self.numeros_grafica.iter().for_each(|val| {
                line_points.push([num, *val as f64]);
                num += 1.0;
            });

            // CORREGIDO: primer argumento es el nombre (vacío), segundo los datos
            let lines = Line::new("", line_points.clone())
                .width(2.0)
                .color(Color32::LIGHT_BLUE);
            let points = Points::new("", line_points)
                .radius(4.0)
                .color(Color32::WHITE);

            plot_ui.line(lines);
            plot_ui.points(points);
        });

        ui.add_space(20.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            // --- 2. Lista de Tareas Globales ---
            ui.push_id("global_tasks", |ui| {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("📋 Active Tasks").heading());
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui
                                .button(if self.add_task {
                                    "❌ Cancel"
                                } else {
                                    "➕ New Task"
                                })
                                .clicked()
                            {
                                self.add_task = !self.add_task;
                            }
                        });
                    });

                    ui.separator();

                    if self.add_task {
                        ui.group(|ui| {
                            ui.label("Title:");
                            ui.add(egui::TextEdit::singleline(&mut self.new_task));
                            ui.label("Description:");
                            ui.add(egui::TextEdit::multiline(&mut self.new_task_desc));
                            ui.checkbox(&mut self.new_task_update, "Add to existing past days");

                            if ui.button("Save Task").clicked() && !self.new_task.is_empty() {
                                self.perform_add_task();
                            }
                        });
                        ui.add_space(10.0);
                    }

                    let mut del = None;
                    let mut id_to_del = 0;

                    for (rem_indx, element) in self.json_content.tasks.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.strong(&element.name);
                            if let Some(desc) = &element.description {
                                ui.label(RichText::new(format!("({})", desc)).weak().small());
                            }

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.menu_button("🗑", |ui| {
                                    ui.label("Irreversible!");
                                    if ui
                                        .button(RichText::new("Confirm Delete").color(Color32::RED))
                                        .clicked()
                                    {
                                        del = Some(rem_indx);
                                        id_to_del = element.id;
                                        ui.close(); // CORREGIDO
                                    }
                                });
                            });
                        });
                    }

                    if let Some(index) = del {
                        self.remove_task_global(index, id_to_del);
                    }
                });
            });

            ui.add_space(20.0);

            // --- 3. Botón Añadir Entrada ---
            if self.add_entry {
                ui.group(|ui| {
                    ui.label("Date (YYYY-MM-DD):");
                    ui.add(egui::TextEdit::singleline(&mut self.new_entry_tittle));
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.add_entry = false;
                        }
                        if ui.button("Save Day").clicked() {
                            self.perform_add_entry();
                        }
                    });
                });
            } else {
                if ui.button("📅 Add New Day Entry").clicked() {
                    self.add_entry = true;
                    self.new_entry_tittle = Local::now().format("%Y-%m-%d").to_string();
                }
            }

            ui.add_space(10.0);

            // --- 4. Lista de Días ---
            let mut del_day_index = None;

            for (ind, element) in self.json_content.days.iter_mut().enumerate() {
                ui.push_id(ind, |ui| {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            if self.edit_task.index == ind as i32
                                && self.edit_task.edit == Edit::Tittle
                            {
                                let resp = ui.add(egui::TextEdit::singleline(&mut self.edit));
                                if resp.lost_focus() || ui.input(|i| i.key_pressed(Key::Enter)) {
                                    element.date = self.edit.clone();
                                    self.edit.clear();
                                    self.edit_task = EditDay {
                                        index: -1,
                                        edit: Edit::Null,
                                    };
                                    self.save_file = true;
                                    self.update_graph = true;
                                }
                            } else {
                                if ui.button(RichText::new(&element.date).heading()).clicked() {
                                    self.edit_task = EditDay {
                                        index: ind as i32,
                                        edit: Edit::Tittle,
                                    };
                                    self.edit = element.date.clone();
                                }
                            }

                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.menu_button(
                                    RichText::new("🗑").color(ui.visuals().error_fg_color),
                                    |ui| {
                                        if ui.button("Confirm Delete Day").clicked() {
                                            del_day_index = Some(ind);
                                            ui.close(); // CORREGIDO
                                        }
                                    },
                                );
                            });
                        });

                        ui.separator();

                        for task in &mut element.tasks {
                            let name = self.tasks_hash.get(&task.id);
                            match name {
                                Some(t) => {
                                    if ui.checkbox(&mut task.completed, t).changed() {
                                        self.update_graph = true;
                                        self.save_file = true;
                                    }
                                }
                                None => {
                                    ui.horizontal(|ui| {
                                        if ui.checkbox(&mut task.completed, "").changed() {
                                            self.update_graph = true;
                                            self.save_file = true;
                                        }
                                        ui.label(
                                            RichText::new(format!(
                                                "Unknown Task (ID: {})",
                                                task.id
                                            ))
                                            .italics()
                                            .weak(),
                                        );
                                    });
                                }
                            }
                        }

                        ui.add_space(5.0);
                        ui.separator();

                        ui.horizontal(|ui| {
                            if self.edit_task.index == ind as i32
                                && self.edit_task.edit == Edit::Description
                            {
                                let resp = ui.add(egui::TextEdit::multiline(&mut self.edit));
                                if resp.lost_focus()
                                    || (ui.input(|i| i.pointer.any_click()) && !resp.hovered())
                                {
                                    if !self.edit.trim().is_empty() {
                                        element.notes = Some(self.edit.clone());
                                    } else {
                                        element.notes = None;
                                    }
                                    self.edit.clear();
                                    self.edit_task = EditDay {
                                        index: -1,
                                        edit: Edit::Null,
                                    };
                                    self.save_file = true;
                                }
                            } else {
                                if let Some(notes) = &element.notes {
                                    ui.label(RichText::new(notes).italics());
                                } else {
                                    ui.label(RichText::new("No notes...").weak().small());
                                }

                                if ui.small_button("📝 Edit Notes").clicked() {
                                    self.edit_task = EditDay {
                                        index: ind as i32,
                                        edit: Edit::Description,
                                    };
                                    self.edit = element.notes.clone().unwrap_or_default();
                                }
                            }
                        });
                    });
                });
                ui.add_space(10.0);
            }

            if let Some(i) = del_day_index {
                self.json_content.days.remove(i);
                self.update_graph = true;
                self.save_file = true;
            }
        });

        if self.update_graph {
            self.calculate_stats();
            self.update_graph = false;
        }
        if self.save_file {
            self.save_tasks();
            self.save_file = false;
        }
    }

    fn perform_add_task(&mut self) {
        self.update_graph = true;
        let new_id = self.json_content.top_id + 1;
        let desc = if self.new_task_desc.trim().is_empty() {
            None
        } else {
            Some(self.new_task_desc.clone())
        };

        self.json_content.tasks.push(TaskToDo {
            name: self.new_task.clone(),
            description: desc,
            id: new_id,
        });

        self.tasks_hash.insert(new_id, self.new_task.clone());

        if self.new_task_update {
            for val in &mut self.json_content.days {
                val.tasks.push(TaskCompleted::new(new_id));
            }
        }

        self.new_task.clear();
        self.new_task_desc.clear();
        self.new_task_update = false;
        self.add_task = false;

        self.json_content.top_id += 1;
        self.save_file = true;
    }

    fn remove_task_global(&mut self, index: usize, id: u32) {
        self.json_content.tasks.remove(index);
        self.rebuild_hashmap();

        for day in &mut self.json_content.days {
            if let Some(pos) = day.tasks.iter().position(|t| t.id == id) {
                day.tasks.remove(pos);
            }
        }

        self.save_file = true;
        self.update_graph = true;
    }

    fn perform_add_entry(&mut self) {
        self.save_file = true;
        self.update_graph = true;

        let tasks: Vec<TaskCompleted> = self
            .json_content
            .tasks
            .iter()
            .map(|t| TaskCompleted::new(t.id))
            .collect();

        self.json_content
            .days
            .insert(0, Day::new(&self.new_entry_tittle, tasks));
        self.add_entry = false;
    }

    pub fn save_tasks(&self) {
        if let Ok(mut file) = fs::File::create(&self.path) {
            if let Ok(conts) = serde_json::to_string_pretty(&self.json_content) {
                let _ = file.write_all(conts.as_bytes());
            }
        }
    }
}
