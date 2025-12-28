pub mod enums;

use crate::iconize::{IconManager, IconSelector};
use crate::screens;
use crate::search;
use crate::MShape;

use chrono::prelude::*;
use eframe::egui::{
    Align, Button, Context, Frame, Layout, RichText, ScrollArea, SidePanel, Style, TopBottomPanel,
};
use egui::Vec2;
use egui::{text::LayoutJob, TextFormat, Widget};

use egui::{Id, Popup, PopupCloseBehavior, Sense};
use std::fs;
use std::fs::File;
use std::path::Path;
use std::time::SystemTime;

use self::enums::{LeftTab, SortOrder};
use crate::main_area::content_enum::Content;
use crate::main_area::file_options::file_options;
use crate::main_area::file_tree::FileTree;

pub struct LeftControls {
    pub current_left_tab: LeftTab,
    pub search_string_menu: String,
    pub prev_search_string_menu: String,
    pub search_results: Vec<search::MenuItem>,
    pub regex_search: bool,

    pub file_tree: FileTree,

    // Gestión de iconos
    pub icon_manager: IconManager,
    pub last_vault_path: String,
}

impl Default for LeftControls {
    fn default() -> Self {
        Self {
            current_left_tab: LeftTab::Files,
            search_string_menu: "".to_owned(),
            prev_search_string_menu: "".to_owned(),
            search_results: vec![],
            regex_search: false,
            file_tree: FileTree::default(),
            icon_manager: IconManager::new(),
            last_vault_path: String::new(),
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
        enable_icons: bool,
        icon_selector: &mut IconSelector,
    ) {
        // Carga perezosa de iconos si cambia el vault o está vacío
        if enable_icons && (self.last_vault_path != path || self.icon_manager.icons.is_empty()) {
            self.icon_manager.load_icons(path);
            self.last_vault_path = path.to_string();
        }

        let left_panel = SidePanel::left("buttons left menu")
            .default_width(100.)
            .min_width(100.)
            .max_width(300.);
        left_panel.show_animated(ctx, *colapse, |ui| {
            self.top_panel_menu_left(
                ui,
                path,
                current_file,
                sort_entrys,
                window_size,
                enable_icons,
                icon_selector,
            );
        });
    }

    #[allow(clippy::too_many_arguments)]
    fn top_panel_menu_left(
        &mut self,
        ui: &mut egui::Ui,
        path: &str,
        current_file: &mut String,
        sort_entrys: &bool,
        window_size: &MShape,
        enable_icons: bool,
        icon_selector: &mut IconSelector,
    ) {
        let vault = path;
        TopBottomPanel::top("Left Menu").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                let btn_size = Vec2::new(window_size.btn_size, window_size.btn_size);
                let color = ui
                    .ctx()
                    .style()
                    .visuals
                    .widgets
                    .noninteractive
                    .fg_stroke
                    .color;

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
                            egui::Image::new(egui::include_image!("../../resources/star.svg"))
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
                                    self.file_tree.sort_order == SortOrder::NameAZ,
                                    "File name (A to Z)",
                                )
                                .clicked()
                            {
                                self.file_tree.sort_order = SortOrder::NameAZ;
                                ui.close();
                            }
                            if ui
                                .selectable_label(
                                    self.file_tree.sort_order == SortOrder::NameZA,
                                    "File name (Z to A)",
                                )
                                .clicked()
                            {
                                self.file_tree.sort_order = SortOrder::NameZA;
                                ui.close();
                            }

                            ui.separator();

                            if ui
                                .selectable_label(
                                    self.file_tree.sort_order == SortOrder::ModifiedNewOld,
                                    "Modified time (new to old)",
                                )
                                .clicked()
                            {
                                self.file_tree.sort_order = SortOrder::ModifiedNewOld;
                                ui.close();
                            }
                            if ui
                                .selectable_label(
                                    self.file_tree.sort_order == SortOrder::ModifiedOldNew,
                                    "Modified time (old to new)",
                                )
                                .clicked()
                            {
                                self.file_tree.sort_order = SortOrder::ModifiedOldNew;
                                ui.close();
                            }

                            ui.separator();

                            if ui
                                .selectable_label(
                                    self.file_tree.sort_order == SortOrder::CreatedNewOld,
                                    "Created time (new to old)",
                                )
                                .clicked()
                            {
                                self.file_tree.sort_order = SortOrder::CreatedNewOld;
                                ui.close();
                            }
                            if ui
                                .selectable_label(
                                    self.file_tree.sort_order == SortOrder::CreatedOldNew,
                                    "Created time (old to new)",
                                )
                                .clicked()
                            {
                                self.file_tree.sort_order = SortOrder::CreatedOldNew;
                                ui.close();
                            }
                        });
                });
            });
        });

        if self.current_left_tab == LeftTab::Files {
            let scrolling_files = ScrollArea::vertical();
            scrolling_files.show(ui, |ui| {
                self.file_tree.render(
                    ui,
                    path,
                    current_file,
                    vault,
                    sort_entrys,
                    enable_icons,
                    &mut self.icon_manager,
                    icon_selector,
                    0,
                );

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
                                self.file_tree.menu_error = format!("Move error: {}", e);
                            } else {
                                // Update icons
                                let old_rel = Path::new(source_str).strip_prefix(path).map(|p| p.to_string_lossy().replace('\\', "/")).unwrap_or_else(|_| source_str.to_string());
                                let new_rel = target_path.strip_prefix(path).map(|p| p.to_string_lossy().replace('\\', "/")).unwrap_or_else(|_| target_path.to_string_lossy().to_string());
                                self.icon_manager.rename_icon(path, &old_rel, &new_rel);

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
                                color: ui.ctx().style().visuals.selection.stroke.color,
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
        command_palette: &mut crate::command_palette::CommandPalette,
    ) {
        let left_panel = SidePanel::left("buttons left")
            .resizable(false)
            .default_width(1.);
        let space = window_size.height / 55.;
        let btn_size = Vec2::new(window_size.btn_size, window_size.btn_size);
        let color = ctx.style().visuals.widgets.noninteractive.fg_stroke.color;
        left_panel.show(ctx, |ui| {
            ui.add_space(5.);
            ui.set_max_width(window_size.btn_size);
            ui.vertical(|ui| {
                if ui
                    .add(Button::image(
                        egui::Image::new(egui::include_image!(
                            "../../resources/fold-horizontal.svg"
                        ))
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
                        egui::Image::new(egui::include_image!(
                            "../../resources/calendar-check.svg"
                        ))
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
                    command_palette.open();
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
                        egui::Image::new(egui::include_image!(
                            "../../resources/square-check-big.svg"
                        ))
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