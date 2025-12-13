use crate::screens;
use crate::search;
use crate::MShape;
use std::io::Write;

use eframe::egui::{
    Align, Button, Context, Frame, Layout, RichText, ScrollArea, SelectableLabel, SidePanel, Style,
    TopBottomPanel,
};
use egui::Vec2;
use egui::{Id, Popup, PopupCloseBehavior};
use json::JsonValue;

use chrono::prelude::*;
use egui::{text::LayoutJob, Color32, TextFormat, Widget};
use std::fs;
use std::fs::File;
use std::path::Path;
use yaml_rust::{YamlEmitter, YamlLoader};

#[derive(PartialEq)]
pub enum LeftTab {
    Files,
    Starred,
    Search,
}

#[derive(PartialEq)]
pub enum Content {
    Edit,
    View,
    NewFile,
    NewTask,
    Graph,
    Blank,
}

pub struct LeftControls {
    current_left_tab: LeftTab,
    search_string_menu: String,
    prev_search_string_menu: String,
    search_results: Vec<search::MenuItem>,
    regex_search: bool,

    //right_collpased:bool,
    //starred_image: RetainedImage,
    rename: String,
    renaming_path: Option<String>,
    menu_error: String,
}

impl Default for LeftControls {
    fn default() -> Self {
        Self {
            current_left_tab: LeftTab::Files,
            rename: "".to_owned(),
            renaming_path: None,
            menu_error: "".to_owned(),
            search_string_menu: "".to_owned(),
            prev_search_string_menu: "".to_owned(),
            search_results: vec![],
            regex_search: false,
        }
    }
}

impl LeftControls {
    pub fn left_side_menu(
        &mut self,
        ctx: &Context,
        colapse: &bool,
        path: &str,
        current_file: &mut String,
        sort_entrys: &bool,
        window_size: &MShape,
    ) {
        let left_panel = SidePanel::left("buttons left menu")
            .default_width(100.)
            .min_width(100.)
            .max_width(300.);
        left_panel.show_animated(ctx, *colapse, |ui| {
            self.top_panel_menu_left(ui, path, current_file, sort_entrys, window_size);
        });
    }

    fn top_panel_menu_left(
        &mut self,
        ui: &mut egui::Ui,
        path: &str,
        current_file: &mut String,
        sort_entrys: &bool,
        window_size: &MShape,
    ) {
        let vault = path;
        TopBottomPanel::top("Left Menu").show_inside(ui, |ui| {
            ui.with_layout(Layout::left_to_right(Align::Min), |ui| {
                let btn_size = Vec2::new(window_size.btn_size, window_size.btn_size);
                let color = Color32::BLACK;
                if ui
                    .add_sized(
                        btn_size.clone(),
                        Button::image(
                            egui::Image::new(egui::include_image!("../resources/folder.svg"))
                                .tint(color)
                                .fit_to_exact_size(btn_size.clone()),
                        ),
                    )
                    .clicked()
                {
                    self.current_left_tab = LeftTab::Files;
                }
                if ui
                    .add_sized(
                        btn_size.clone(),
                        Button::image(
                            egui::Image::new(egui::include_image!("../resources/search.svg"))
                                .tint(color)
                                .fit_to_exact_size(btn_size.clone()),
                        ),
                    )
                    .clicked()
                {
                    self.current_left_tab = LeftTab::Search;
                }
                if ui
                    .add_sized(
                        btn_size.clone(),
                        Button::image(
                            egui::Image::new(egui::include_image!(
                                "../resources/square-check-big.svg"
                            ))
                            .fit_to_exact_size(btn_size.clone())
                            .tint(color),
                        ),
                    )
                    .clicked()
                {
                    self.current_left_tab = LeftTab::Starred;
                }
            });
        });

        if self.current_left_tab == LeftTab::Files {
            let scrolling_files = ScrollArea::vertical();
            scrolling_files.show(ui, |ui| {
                self.render_files(ui, path, current_file, vault, sort_entrys);
            });
        } else if self.current_left_tab == LeftTab::Search {
            ui.text_edit_singleline(&mut self.search_string_menu);
            ui.checkbox(&mut self.regex_search, "regex");
            if self.search_string_menu != self.prev_search_string_menu {
                if self.regex_search {
                    self.search_results = search::check_dir_regex(path, &self.search_string_menu);
                    self.prev_search_string_menu = self.search_string_menu.to_string();
                } else {
                    self.search_results = search::check_dir(path, &self.search_string_menu);
                    self.prev_search_string_menu = self.search_string_menu.to_string();
                }
            }
            let style_frame = Style::default();
            let frame = Frame::group(&style_frame);
            if self.search_string_menu.len() < 1 {
                self.search_results = vec![];
            }
            let scrolling_search = ScrollArea::vertical();
            scrolling_search.show(ui, |ui| {
                for i in &self.search_results {
                    frame.show(ui, |ui| {
                        let mut title = LayoutJob::default();
                        title.append(
                            &i.path.strip_prefix(&path).unwrap(),
                            0.0,
                            TextFormat {
                                color: Color32::RED,
                                ..Default::default()
                            },
                        );
                        ui.label(title);
                        ui.label(&i.text);
                        if ui.button("open file").clicked() {
                            *current_file = String::from(&i.path);
                        };
                    });
                }
            });
        } else if self.current_left_tab == LeftTab::Starred {
            let contents = match fs::read_to_string(format!("{}/.obsidian/starred.json", path)) {
                Ok(x) => x.as_str().to_owned(),
                _ => {
                    ui.label("No starred file found!");
                    return;
                }
            };
            let parsed = json::parse(&contents).unwrap();
            for (_key, value) in parsed.entries() {
                for i in 0..value.len() {
                    let text = parsed["items"][i]["path"].as_str().unwrap();
                    let full_path = format!("{}/{}", path, text);
                    if full_path == current_file.as_str() {
                        ui.label(RichText::new(text).color(ui.style().visuals.selection.bg_fill));
                    } else {
                        let btn = Button::new(text).frame(false);
                        if btn.ui(ui).clicked() {
                            *current_file = Path::new(&full_path).to_str().unwrap().to_owned();
                        }
                    }
                }
            }
        }
    }

    fn render_files(
        &mut self,
        ui: &mut egui::Ui,
        path: &str,
        current_file: &mut String,
        vault: &str,
        sort_entrys: &bool,
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
            entrys_vec.push(entry.unwrap().path().to_str().unwrap().to_string());
        }
        if *sort_entrys {
            entrys_vec.sort(); //Stop sorting every frame
        }

        for file_location in entrys_vec {
            let file_name = Path::new(&file_location)
                .file_name()
                .expect("No fails")
                .to_str()
                .unwrap();

            if Path::new(&file_location).is_dir() {
                let col = egui::containers::collapsing_header::CollapsingHeader::new(file_name);
                col.show(ui, |ui| {
                    self.render_files(ui, &file_location, current_file, vault, sort_entrys);
                });
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

                    let btn = Button::selectable(is_selected, file_name);

                    let response = ui.add(btn);
                    let popup_id = Id::new("file_menu").with(&file_location);

                    Popup::context_menu(&response)
                        .id(popup_id)
                        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
                        .show(|ui| {
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

    pub fn left_side_settings(
        &self,
        ctx: &Context,
        colapse: &mut bool,
        vault: &mut String,
        current_file: &mut String,
        current_window: &mut screens::Screen,
        content: &mut Content,
        window_size: &MShape,
    ) {
        let left_panel = SidePanel::left("buttons left")
            .resizable(false)
            .default_width(1.);
        let space = window_size.height / 55.;
        let btn_size = Vec2::new(window_size.btn_size, window_size.btn_size);
        let color = Color32::BLACK;
        left_panel.show(ctx, |ui| {
            ui.add_space(5.);
            ui.set_max_width(window_size.btn_size);
            ui.vertical(|ui| {
                if ui
                    .add(Button::image(
                        egui::Image::new(egui::include_image!("../resources/fold-horizontal.svg"))
                            .fit_to_exact_size(btn_size.clone())
                            .tint(color),
                    ))
                    .clicked()
                {
                    *colapse = !*colapse;
                }
                ui.add_space(space);
                if ui
                    .add(egui::Button::image(
                        egui::Image::new(egui::include_image!("../resources/file-search.svg"))
                            .fit_to_exact_size(btn_size.clone())
                            .tint(color),
                    ))
                    .on_hover_text("Switcher")
                    .clicked()
                {
                    println!("switcher")
                } //quick switcher
                ui.add_space(space);
                if ui
                    .add(egui::Button::image(
                        egui::Image::new(egui::include_image!("../resources/graph.svg"))
                            .fit_to_exact_size(btn_size.clone())
                            .tint(color),
                    ))
                    .on_hover_text("Graph")
                    .clicked()
                {
                    *content = Content::Graph;
                } //graph
                ui.add_space(space);
                if ui
                    .add(egui::Button::image(
                        egui::Image::new(egui::include_image!("../resources/canvas.svg"))
                            .fit_to_exact_size(btn_size.clone())
                            .tint(color),
                    ))
                    .on_hover_text("Canvas")
                    .clicked()
                {
                    println!("canvas")
                } //canvas
                ui.add_space(space);
                if ui
                    .add(egui::Button::image(
                        egui::Image::new(egui::include_image!("../resources/calendar-check.svg"))
                            .fit_to_exact_size(btn_size.clone())
                            .tint(color),
                    ))
                    .on_hover_text("Daily note")
                    .clicked()
                {
                    Self::create_date_file(vault, current_file);
                } //note
                ui.add_space(space);
                if ui
                    .add(egui::Button::image(
                        egui::Image::new(egui::include_image!("../resources/terminal.svg"))
                            .fit_to_exact_size(btn_size.clone())
                            .tint(color),
                    ))
                    .on_hover_text("Command Palette")
                    .clicked()
                {
                    println!("command palette")
                } //palette
                ui.add_space(space);
                if ui
                    .add(egui::Button::image(
                        egui::Image::new(egui::include_image!("../resources/new_file.svg"))
                            .fit_to_exact_size(btn_size.clone())
                            .tint(color),
                    ))
                    .on_hover_text("New File")
                    .clicked()
                {
                    *content = Content::NewFile;
                }

                ui.with_layout(Layout::bottom_up(Align::Max), |ui| {
                    ui.add_space(5.);
                    if ui
                        .add(egui::Button::image(
                            egui::Image::new(egui::include_image!("../resources/cog.svg"))
                                .fit_to_exact_size(btn_size.clone())
                                .tint(color),
                        ))
                        .on_hover_text("Configuration")
                        .clicked()
                    {
                        *current_window = screens::Screen::Configuracion;
                    }
                    ui.add_space(5.);
                    if ui
                        .add(egui::Button::image(
                            egui::Image::new(egui::include_image!(
                                "../resources/badge-question-mark.svg"
                            ))
                            .fit_to_exact_size(btn_size.clone())
                            .tint(color),
                        ))
                        .on_hover_text("Help")
                        .clicked()
                    {
                        println!("help")
                    }
                });
            });
        });
    }

    fn create_date_file(path: &String, current_file: &mut String) {
        let date = Local::now().format("%Y-%m-%d").to_string();
        let file_name = format!("{}/{}.md", path, date);
        if Path::new(&file_name).exists() {
            *current_file = file_name.to_string();
        } else {
            File::create(&file_name).expect("Unable to create file");
            *current_file = file_name.to_string();
        }
    }
}

pub fn create_metadata(metadata: String, ui: &mut egui::Ui) {
    let result = YamlLoader::load_from_str(&metadata);
    let Ok(docs) = result else {
        egui::Frame::group(ui.style())
            .fill(ui.visuals().faint_bg_color)
            .corner_radius(5.0)
            .inner_margin(10.0)
            .stroke(egui::Stroke::new(1.0, Color32::RED))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label("Bad Formatting! :(");
            });
        ui.add_space(5.0);
        return;
    };
    let metadata_parsed = &docs[0];
    let mut job = LayoutJob::default();

    let mut out_str = String::new();
    let mut emitter = YamlEmitter::new(&mut out_str);
    emitter.dump(metadata_parsed).unwrap();
    out_str.split("\n").skip(1).for_each(|s| {
        if s.as_bytes()[s.len() - 1] == 58 {
            job.append(
                &(s.to_owned() + "\n"),
                0.0,
                TextFormat {
                    color: Color32::GRAY,
                    ..Default::default()
                },
            )
        } else if s.as_bytes()[0] == 32 {
            job.append(
                &(s.to_owned() + "\n"),
                0.0,
                TextFormat {
                    color: Color32::WHITE,
                    ..Default::default()
                },
            )
        } else {
            let mut splitted = s.split(" ");
            let mut content: &str;
            let mut text = splitted.next();
            match text {
                Some(x) => content = x,
                None => content = "Error parsing",
            }
            job.append(
                content,
                0.0,
                TextFormat {
                    color: Color32::GRAY,
                    ..Default::default()
                },
            );
            text = splitted.next();
            match text {
                Some(x) => content = x,
                None => content = "Error parsing",
            }
            job.append(
                &format!("{}\n", content),
                0.0,
                TextFormat {
                    color: Color32::WHITE,
                    ..Default::default()
                },
            );
        }
    });
    egui::Frame::group(ui.style())
        .fill(ui.visuals().faint_bg_color)
        .corner_radius(5.0)
        .inner_margin(10.0)
        .stroke(egui::Stroke::new(
            1.0,
            ui.visuals().widgets.noninteractive.bg_stroke.color,
        ))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(job);
        });
}

fn file_options(
    ui: &mut egui::Ui,
    s: &str,
    path: &str,
    rename: &mut String,
    renaming_path: &mut Option<String>,
    error: &mut String,
    vault: &str,
) {
    let stared_path = format!("{}/.obsidian/starred.json", vault);

    ui.label(RichText::new(&*error).color(Color32::RED));
    let copy = egui::Button::new("Copy file").frame(false);
    let star = egui::Button::new("Star this file").frame(false);
    let path_s = Path::new(s).file_name().unwrap();

    ui.label("Move");

    if ui.add(copy).clicked() {
        let tmp = s.to_owned() + ".copy";
        let s_copy = Path::new(&tmp);
        let copy = fs::copy(s, &s_copy);
        match copy {
            Ok(_) => {
                *error = String::new();
                ui.close();
            }
            Err(r) => *error = r.to_string(),
        }
    }

    if ui.add(star).clicked() {
        let nw_json = object! {
            "type":"file",
            "title":Path::new(path_s).file_stem().unwrap().to_str().unwrap(),
            "path":"testi"
        };
        if Path::new(&stared_path).exists() {
            let contents =
                fs::read_to_string(&stared_path).expect("Should have been able to read the file");
            let mut parsed = json::parse(&contents).unwrap();
            let json_arr: &mut JsonValue = &mut parsed["items"];
            json_arr.push(nw_json).unwrap();
            let mut f = std::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(stared_path)
                .unwrap();
            f.write_all(parsed.pretty(2).as_bytes()).unwrap();
            f.flush().unwrap();
        } else {
            let file = File::create(stared_path);
            match file {
                Ok(mut w) => {
                    let text = format!("{{ items:[{}] }}", nw_json.dump());
                    match w.write(text.as_bytes()) {
                        Ok(_) => *error = String::new(),
                        Err(r) => *error = r.to_string(),
                    }
                }
                Err(r) => *error = r.to_string(),
            }
        }
        ui.close();
    }

    if ui.button("Rename").clicked() {
        *renaming_path = Some(s.to_string());
        *rename = Path::new(s)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        ui.close();
    }

    ui.separator();

    let id = ui.make_persistent_id(format!("del_{}", s));
    let mut is_confirming: bool = ui.data(|d| d.get_temp(id).unwrap_or(false));

    if is_confirming {
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("Are you sure?").strong().color(Color32::RED));
            ui.label(RichText::new(path_s.to_str().unwrap()).italics().weak());
            ui.add_space(5.);

            ui.horizontal(|ui| {
                // Cancelar
                if ui.button("No").clicked() {
                    is_confirming = false;
                    ui.data_mut(|d| d.insert_temp(id, false));
                }

                let btn_yes = egui::Button::new(RichText::new("Yes, Delete").color(Color32::WHITE))
                    .fill(Color32::RED);

                if ui.add(btn_yes).clicked() {
                    let delete = fs::remove_file(s);
                    match delete {
                        Ok(_) => {
                            *error = String::new();
                            ui.data_mut(|d| d.remove_temp::<bool>(id));
                            ui.close();
                        }
                        Err(r) => {
                            *error = r.to_string();
                        }
                    }
                }
            });
        });
    } else {
        let btn_delete =
            Button::selectable(false, RichText::new("Delete file").color(Color32::RED));

        if ui.add(btn_delete).clicked() {
            is_confirming = true;
            ui.data_mut(|d| d.insert_temp(id, true));
        }
    }
}
