pub mod enums;

use crate::screens;
use crate::search;
use crate::MShape;

use std::time::SystemTime;
use eframe::egui::{
    Align, Button, Context, Frame, Layout, RichText, ScrollArea, SidePanel, Style, TopBottomPanel,
};
use egui::Vec2;
use egui::{Id, Popup, PopupCloseBehavior};
use chrono::prelude::*;
use egui::{text::LayoutJob, Color32, TextFormat, Widget}; // `Widget` is not used directly
use std::fs;
use std::fs::File;
use std::path::Path;

use crate::main_area::content_enum::Content;
use crate::main_area::file_options::file_options;
use self::enums::{LeftTab, SortOrder};

pub struct LeftControls {
    pub current_left_tab: LeftTab,
    pub search_string_menu: String,
    pub prev_search_string_menu: String,
    pub search_results: Vec<search::MenuItem>,
    pub regex_search: bool,

    pub sort_order: SortOrder,

    pub rename: String,
    pub renaming_path: Option<String>,
    pub menu_error: String,
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
            sort_order: SortOrder::NameAZ,
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
            ui.horizontal(|ui| {
                let btn_size = Vec2::new(window_size.btn_size, window_size.btn_size);
                let color = Color32::BLACK;

                if ui
                    .add_sized(
                        btn_size.clone(),
                        Button::image(
                            egui::Image::new(egui::include_image!("../../resources/folder.svg"))
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
                            egui::Image::new(egui::include_image!("../../resources/search.svg"))
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
                                "../../resources/square-check-big.svg"
                            ))
                            .fit_to_exact_size(btn_size.clone())
                            .tint(color),
                        ),
                    )
                    .clicked()
                {
                    self.current_left_tab = LeftTab::Starred;
                }

                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let sort_btn_response = ui
                        .add_sized(
                            btn_size.clone(),
                            Button::image(
                                egui::Image::new(egui::include_image!(
                                    "../../resources/arrow-down-wide-narrow.svg"
                                ))
                                .fit_to_exact_size(btn_size.clone())
                                .tint(color),
                            ),
                        )
                        .on_hover_text("Sort files");

                    Popup::menu(&sort_btn_response)
                        .id(Id::new("sort_menu_popup"))
                        .show(|ui| {
                            ui.set_min_width(150.0);
                            ui.label(RichText::new("Sort options").strong().size(12.0));
                            ui.separator();

                            if ui
                                .selectable_label(
                                    self.sort_order == SortOrder::NameAZ,
                                    "File name (A to Z)",
                                )
                                .clicked()
                            {
                                self.sort_order = SortOrder::NameAZ;
                                ui.close();
                            }
                            if ui
                                .selectable_label(
                                    self.sort_order == SortOrder::NameZA,
                                    "File name (Z to A)",
                                )
                                .clicked()
                            {
                                self.sort_order = SortOrder::NameZA;
                                ui.close();
                            }

                            ui.separator();

                            if ui
                                .selectable_label(
                                    self.sort_order == SortOrder::ModifiedNewOld,
                                    "Modified time (new to old)",
                                )
                                .clicked()
                            {
                                self.sort_order = SortOrder::ModifiedNewOld;
                                ui.close();
                            }
                            if ui
                                .selectable_label(
                                    self.sort_order == SortOrder::ModifiedOldNew,
                                    "Modified time (old to new)",
                                )
                                .clicked()
                            {
                                self.sort_order = SortOrder::ModifiedOldNew;
                                ui.close();
                            }

                            ui.separator();

                            if ui
                                .selectable_label(
                                    self.sort_order == SortOrder::CreatedNewOld,
                                    "Created time (new to old)",
                                )
                                .clicked()
                            {
                                self.sort_order = SortOrder::CreatedNewOld;
                                ui.close();
                            }
                            if ui
                                .selectable_label(
                                    self.sort_order == SortOrder::CreatedOldNew,
                                    "Created time (old to new)",
                                )
                                .clicked()
                            {
                                self.sort_order = SortOrder::CreatedOldNew;
                                ui.close();
                            }
                        });
                });
            });
        });

        if self.current_left_tab == LeftTab::Files {
            let scrolling_files = ScrollArea::vertical();
            scrolling_files.show(ui, |ui| {
                self.render_files(ui, path, current_file, vault, sort_entrys);

                let available_size = ui.available_size();
                let min_height = if available_size.y < 50.0 {
                    50.0
                } else {
                    available_size.y
                };

                let (_, dropped_payload) = ui.dnd_drop_zone::<String, ()>(Frame::NONE, |ui| {
                    ui.set_min_size(Vec2::new(ui.available_width(), min_height));
                });

                if let Some(source_path_arc) = dropped_payload {
                    let source_str: &str = &source_path_arc;
                    let root_path = Path::new(path);

                    if let Some(file_name) = Path::new(source_str).file_name() {
                        let target_path = root_path.join(file_name);

                        if target_path != Path::new(source_str) {
                            if let Err(e) = fs::rename(source_str, &target_path) {
                                self.menu_error = format!("Move error: {}", e);
                            } else {
                                if *current_file == source_str {
                                    *current_file = target_path.to_str().unwrap().to_string();
                                }
                            }
                        }
                    }
                }
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

            if Path::new(&file_location).is_dir() {
                let count = fs::read_dir(&file_location).map(|i| i.count()).unwrap_or(0);
                let folder_label = format!("{}  [{}]", file_name, count);

                let header =
                    egui::containers::collapsing_header::CollapsingHeader::new(folder_label);

                let response = header.show(ui, |ui| {
                    self.render_files(ui, &file_location, current_file, vault, sort_entrys);
                });

                let header_response = response.header_response;

                if header_response.dnd_hover_payload::<String>().is_some() {
                    ui.painter().rect_stroke(
                        header_response.rect,
                        2.0,
                        egui::Stroke::new(2.0, Color32::from_rgb(255, 165, 0)),
                        egui::StrokeKind::Middle,
                    );
                }

                if let Some(source_path) = header_response.dnd_release_payload::<String>() {
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
                        let label = egui::Button::selectable(is_selected, file_name)
                            .sense(egui::Sense::hover());
                        ui.add(label)
                    });

                    let response = dnd_response.response.interact(egui::Sense::click());

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
        tabs: &mut crate::tabs::Tabs,
        tabs_counter: &mut usize,
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
                        egui::Image::new(egui::include_image!("../../resources/fold-horizontal.svg"))
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
                        egui::Image::new(egui::include_image!("../../resources/file-search.svg"))
                            .fit_to_exact_size(btn_size.clone())
                            .tint(color),
                    ))
                    .on_hover_text("Switcher")
                    .clicked()
                {
                    println!("switcher")
                }
                ui.add_space(space);
                if ui
                    .add(egui::Button::image(
                        egui::Image::new(egui::include_image!("../../resources/graph.svg"))
                            .fit_to_exact_size(btn_size.clone())
                            .tint(color),
                    ))
                    .on_hover_text("Graph")
                    .clicked()
                {
                    tabs.add_tab(crate::tabs::Tabe::new_graph(*tabs_counter, vault));
                    *tabs_counter += 1;
                }
                ui.add_space(space);
                if ui
                    .add(egui::Button::image(
                        egui::Image::new(egui::include_image!("../../resources/canvas.svg"))
                            .fit_to_exact_size(btn_size.clone())
                            .tint(color),
                    ))
                    .on_hover_text("Canvas")
                    .clicked()
                {
                    println!("canvas")
                }
                ui.add_space(space);
                if ui
                    .add(egui::Button::image(
                        egui::Image::new(egui::include_image!("../../resources/calendar-check.svg"))
                            .fit_to_exact_size(btn_size.clone())
                            .tint(color),
                    ))
                    .on_hover_text("Daily note")
                    .clicked()
                {
                    Self::create_date_file(vault, current_file);
                }
                ui.add_space(space);
                if ui
                    .add(egui::Button::image(
                        egui::Image::new(egui::include_image!("../../resources/terminal.svg"))
                            .fit_to_exact_size(btn_size.clone())
                            .tint(color),
                    ))
                    .on_hover_text("Command Palette")
                    .clicked()
                {
                    println!("command palette")
                }
                ui.add_space(space);
                if ui
                    .add(egui::Button::image(
                        egui::Image::new(egui::include_image!("../../resources/new_file.svg"))
                            .fit_to_exact_size(btn_size.clone())
                            .tint(color),
                    ))
                    .on_hover_text("New File")
                    .clicked()
                {
                    *content = Content::NewFile;
                }
                ui.add_space(space);
                if ui
                    .add(egui::Button::image(
                        egui::Image::new(egui::include_image!("../../resources/tasks.png"))
                            .fit_to_exact_size(btn_size.clone())
                            .tint(color),
                    ))
                    .on_hover_text("New Task")
                    .clicked()
                {
                    *content = Content::NewTask;
                }

                ui.with_layout(Layout::bottom_up(Align::Max), |ui| {
                    ui.add_space(5.);
                    if ui
                        .add(egui::Button::image(
                            egui::Image::new(egui::include_image!("../../resources/cog.svg"))
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
                                "../../resources/badge-question-mark.svg"
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