use chrono::Local;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;

use egui::*;
use egui_plot::{GridMark, Line, Plot, Points};
use std::ops::RangeInclusive;

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
    let data: TasksFile = match serde_json::from_str(&data) {
        Ok(t) => t,
        Err(_) => TasksFile::default(),
    };
    data
}

pub struct TasksGui {
    json_content: TasksFile,
    path: String,
    numeros_grafica: Vec<u16>,
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
    prom: f32,
}

impl Default for TasksGui {
    fn default() -> Self {
        Self {
            save_file: false,
            add_entry: false,
            update_graph: false,
            json_content: TasksFile {
                tasks: vec![],
                days: vec![],
                top_id: 0,
            },
            path: String::new(),
            numeros_grafica: vec![],
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
        for element in &self.json_content.tasks {
            self.tasks_hash.insert(element.id, element.name.clone());
        }
    }

    pub fn set_path(&mut self, path: &str) {
        if path != &self.path {
            self.path = String::from(path);
            let mut prom: u16 = 0;
            let mut tot_num = 0.0;
            self.set_tasks(load_tasks(&self.path));
            for element in &self.json_content.days {
                let mut tot: u16 = 0;
                element
                    .tasks
                    .iter()
                    .for_each(|val| tot += val.completed as u16);
                prom += tot;
                self.numeros_grafica.push(tot);
                tot_num += 1.0;
            }
            self.prom = prom as f32 / tot_num;
            self.numeros_grafica.reverse();
            self.update_graph();
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        let x_fmt = |x: GridMark, _range: &RangeInclusive<f64>| format!("Day {}", x.value);

        let markers_plot = Plot::new("Graph")
            .height(200.0)
            .x_axis_formatter(x_fmt)
            .data_aspect(0.70)
            .auto_bounds(true)
            .clamp_grid(true);
        ui.label(&format!("{} completed tasks a day done", self.prom));
        markers_plot.show(ui, |plot_ui| {
            let mut num = 0.0;
            let mut line_points = vec![];
            self.numeros_grafica.iter().for_each(|val| {
                line_points.push([num, *val as f64]);
                num += 1.0;
            });
            let lines = Line::new("", line_points.clone()).width(5.0).fill(0.0);
            let points = Points::new("", line_points).radius(4.0);
            plot_ui.line(lines);
            plot_ui.points(points);
        });

        ui.add_space(10.0);
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.group(|ui| {
                ui.label(
                    RichText::new("Tasks")
                        .color(ui.visuals().strong_text_color())
                        .size(20.0),
                );
                let mut del = 0;
                let mut del_bool = false;
                let mut id = 0;
                for (rem_indx, element) in self.json_content.tasks.iter().enumerate() {
                    ui.horizontal_top(|ui| {
                        ui.heading(RichText::new(&element.name));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            if ui.button("X").clicked() {
                                del = rem_indx;
                                del_bool = true;
                                id = element.id;
                                self.save_file = true;
                            }
                        });
                    });
                    match &element.description {
                        Some(expr) => {
                            ui.label(expr);
                        }
                        None => {}
                    };
                    ui.separator();
                }
                if del_bool {
                    self.json_content.tasks.remove(del);
                    for i in &mut self.json_content.days {
                        for (ind, j) in i.tasks.iter_mut().enumerate() {
                            if j.id == id {
                                i.tasks.remove(ind);
                                break;
                            }
                        }
                    }
                }
                if self.add_task {
                    ui.label("Title:");
                    ui.add(egui::TextEdit::singleline(&mut self.new_task));
                    ui.label("Description:");
                    ui.add(egui::TextEdit::multiline(&mut self.new_task_desc));
                    ui.checkbox(&mut self.new_task_update, "Add to previous tasks");
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.new_task = String::new();
                            self.new_task_desc = String::new();
                            self.add_task = false;
                        }
                        if ui.button("Add").clicked() {
                            let new_id = self.json_content.top_id + 1;
                            let desc: Option<String> = if self.new_task_desc.is_empty() {
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
                                    val.tasks.push(TaskCompleted {
                                        completed: false,
                                        id: new_id,
                                    });
                                }
                            }
                            self.new_task = String::new();
                            self.new_task_update = false;
                            self.new_task_desc = String::new();
                            self.add_task = false;
                            self.json_content.top_id += 1;
                            self.save_file = true;
                        }
                    });
                } else if ui.button("Add Task").clicked() {
                    self.add_task = true;
                }
            });

            ui.add_space(10.0);
            if self.add_entry {
                ui.group(|ui| {
                    ui.label("Tittle");
                    ui.add(egui::TextEdit::singleline(&mut self.new_entry_tittle));
                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.add_entry = false;
                        }
                        if ui.button("Add").clicked() {
                            self.save_file = true;
                            let mut tasks = vec![];
                            for i in &self.json_content.tasks {
                                tasks.push(TaskCompleted::new(i.id));
                            }
                            self.json_content
                                .days
                                .insert(0, Day::new(&self.new_entry_tittle, tasks));
                            self.add_entry = false;
                        }
                    });
                });
            } else if ui.button("Add Entry").clicked() {
                self.add_entry = true;
                let date = Local::now().format("%Y-%m-%d").to_string();
                self.new_entry_tittle = date;
                self.save_file = true;
            }
            let mut del_ind = 0;
            let mut del_ind_bool = false;
            for (ind, element) in self.json_content.days.iter_mut().enumerate() {
                let btn = egui::Button::new(&element.date).frame(false);
                ui.group(|ui| {
                    ui.horizontal_top(|ui| {
                        if self.edit_task.index == ind as i32 && self.edit_task.edit == Edit::Tittle
                        {
                            if ui
                                .add(egui::TextEdit::singleline(&mut self.edit))
                                .lost_focus()
                            {
                                self.edit_task = EditDay {
                                    index: -1,
                                    edit: Edit::Null,
                                };
                                element.date = String::from(&self.edit);
                                self.edit = String::new();
                            }
                        } else if ui.add(btn).clicked() {
                            self.edit_task = EditDay {
                                index: ind as i32,
                                edit: Edit::Tittle,
                            };
                            self.edit = element.date.clone();
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            if ui.button("X").clicked() {
                                del_ind_bool = true;
                                del_ind = ind;
                                self.update_graph = true;
                                self.save_file = true;
                            }
                        });
                    });
                    ui.separator();
                    for task in &mut element.tasks {
                        let name = self.tasks_hash.get(&task.id);
                        match name {
                            Some(t) => {
                                if ui
                                    .add(egui::Checkbox::new(&mut task.completed, t))
                                    .changed()
                                {
                                    self.update_graph = true;
                                    self.save_file = true;
                                }
                            }
                            None => {
                                if ui
                                    .add(egui::Checkbox::new(&mut task.completed, "Task Not Found"))
                                    .changed()
                                {
                                    self.update_graph = true;
                                    self.save_file = true;
                                }
                            }
                        }
                    }
                    ui.add_space(10.0);
                    if self.edit_task.index == ind as i32
                        && self.edit_task.edit == Edit::Description
                    {
                        let resp = ui.add(egui::TextEdit::multiline(&mut self.edit));
                        if resp.lost_focus() || resp.clicked_elsewhere() {
                            self.edit_task = EditDay {
                                index: -1,
                                edit: Edit::Null,
                            };
                            if !self.edit.is_empty() {
                                element.notes = Some(String::from(&self.edit));
                                self.edit = String::new();
                            } else {
                                element.notes = None;
                            }
                            self.save_file = true;
                        }
                    } else if self.edit_task.edit != Edit::Description {
                        match &element.notes {
                            Some(expr) => {
                                ui.label(expr);
                            }
                            None => {}
                        };
                        if ui.button("Edit desc").clicked() {
                            self.edit_task = EditDay {
                                index: ind as i32,
                                edit: Edit::Description,
                            };
                            self.edit = element.notes.clone().unwrap_or_default();
                        }
                    }
                });
                ui.add_space(10.0);
            }
            if del_ind_bool {
                self.json_content.days.remove(del_ind);
            }
        });
        if self.update_graph {
            self.update_graph();
            self.update_graph = false;
        }
        if self.save_file {
            self.save_tasks();
            self.save_file = false;
        }
    }

    fn update_graph(&mut self) {
        let mut prom: u16 = 0;
        let mut tot_num = 0.0;
        let mut tmp = vec![];
        for element in &self.json_content.days {
            let mut tot: u16 = 0;
            element
                .tasks
                .iter()
                .for_each(|val| tot += val.completed as u16);
            prom += tot;
            tot_num += 1.0;
            tmp.push(tot);
        }
        tmp.reverse();
        self.numeros_grafica = tmp;
        self.prom = prom as f32 / tot_num;
    }

    pub fn save_tasks(&self) {
        let file = String::from(&self.path);
        let mut file2 = fs::File::create(file).unwrap();
        let conts = serde_json::to_string(&self.json_content).unwrap();
        file2.write_all(conts.as_bytes()).unwrap();
    }
}
