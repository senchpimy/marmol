use crate::excalidraw;
use crate::easy_mark;
use crate::files;
use crate::iconize::{IconManager, IconSource};
use crate::income;
use crate::kanban;

use crate::main_area::content_enum::Content;
use crate::main_area::metadata_renderer::create_metadata;
use crate::tasks;
use egui::Image;
use egui::{Frame, Sense, Ui, WidgetText};
use crate::egui_commonmark::*;
use crate::egui_dock::{DockArea, DockState, NodeIndex, Style, SurfaceIndex, TabViewer};
use egui_extras::{Size, StripBuilder};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::Path;

#[derive(Serialize, Deserialize)]
pub enum TabContent {
    Empty,
    Graph {
        vault_path: String,
        #[serde(skip)]
        state: Option<Box<crate::graph::Graph>>,
    },
    Image(String),
    Income {
        path: String,
        #[serde(skip, default)]
        gui: income::IncomeGui,
    },
    Tasks {
        path: String,
        #[serde(skip, default)]
        gui: tasks::TasksGui,
    },
    Excalidraw {
        path: String,
        #[serde(skip, default)]
        gui: excalidraw::ExcalidrawGui,
    },
    Kanban {
        path: String,
        #[serde(skip, default)]
        gui: kanban::KanbanGui,
    },
    Markdown {
        #[serde(skip, default)]
        editor: easy_mark::EasyMarkEditor,
        #[serde(skip)]
        cache: CommonMarkCache,
    },
}

impl Default for TabContent {
    fn default() -> Self {
        TabContent::Empty
    }
}

impl Clone for TabContent {
    fn clone(&self) -> Self {
        match self {
            TabContent::Empty => TabContent::Empty,
            TabContent::Graph { vault_path, .. } => TabContent::Graph {
                vault_path: vault_path.clone(),
                state: None,
            },
            TabContent::Image(path) => TabContent::Image(path.clone()),
            TabContent::Income { path, .. } => {
                let mut gui = income::IncomeGui::default();
                gui.set_path(path);
                TabContent::Income {
                    path: path.clone(),
                    gui,
                }
            }
            TabContent::Tasks { path, .. } => {
                let mut gui = tasks::TasksGui::default();
                gui.set_path(path);
                TabContent::Tasks {
                    path: path.clone(),
                    gui,
                }
            }
            TabContent::Excalidraw { path, .. } => {
                let mut new_gui = excalidraw::ExcalidrawGui::default();
                new_gui.set_path(path);
                TabContent::Excalidraw {
                    path: path.clone(),
                    gui: new_gui,
                }
            }
            TabContent::Kanban { path, .. } => {
                let mut gui = kanban::KanbanGui::default();
                gui.set_path(path);
                TabContent::Kanban {
                    path: path.clone(),
                    gui,
                }
            }
            TabContent::Markdown { .. } => TabContent::Markdown {
                editor: easy_mark::EasyMarkEditor::default(),
                cache: CommonMarkCache::default(),
            },
        }
    }
}

//type Tabe = String;
#[derive(Serialize, Deserialize, Clone)]
pub struct Tabe {
    pub id: usize,
    pub title: String,
    pub path: String,
    #[serde(default)]
    pub content: TabContent,
    pub ctype: Content,
    #[serde(default)]
    pub history: Vec<String>,
    #[serde(default)]
    pub history_index: usize,
    #[serde(skip)]
    pub is_renaming: bool,
    #[serde(skip)]
    pub rename_buffer: String,
}

impl Tabe {
    fn new(n: usize, path: String) -> Self {
        let title = Path::new(&path)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .into();
        let loaded_content = files::read_file(&path);

        let (initial_ctype, _initial_buffer) = if loaded_content.trim().is_empty() {
            (Content::Edit, loaded_content.clone())
        } else {
            (Content::View, String::new())
        };

        let content = if path.ends_with(".png") || path.ends_with("jpeg") || path.ends_with("jpg") {
            TabContent::Image(path.clone())
        } else if path.ends_with(".inc") {
            let mut gui = income::IncomeGui::default();
            gui.set_path(&path);
            TabContent::Income {
                path: path.clone(),
                gui,
            }
        } else if path.ends_with(".excalidraw.md") {
            let mut gui = excalidraw::ExcalidrawGui::default();
            gui.set_path(&path);
            TabContent::Excalidraw {
                path: path.clone(),
                gui,
            }
        } else if path.ends_with(".graph") {
            let mut gui = tasks::TasksGui::default();
            gui.set_path(&path);
            TabContent::Tasks {
                path: path.clone(),
                gui,
            }
        } else if loaded_content.contains("kanban-plugin: board") {
            let mut gui = kanban::KanbanGui::default();
            gui.set_path(&path);
            TabContent::Kanban {
                path: path.clone(),
                gui,
            }
        } else {
            let mut editor = easy_mark::EasyMarkEditor::default();
            editor.code = loaded_content;
            TabContent::Markdown {
                editor,
                cache: CommonMarkCache::default(),
            }
        };

        Self {
            id: n,
            ctype: initial_ctype,
            title,
            path: path.clone(),
            content,
            history: vec![path],
            history_index: 0,
            is_renaming: false,
            rename_buffer: String::new(),
        }
    }

    pub fn new_graph(n: usize, vault: &str) -> Self {
        Self {
            id: n,
            ctype: Content::Graph,
            title: "Graph".to_string(),
            path: String::new(),
            content: TabContent::Graph {
                vault_path: vault.to_string(),
                state: None,
            },
            history: vec![String::new()],
            history_index: 0,
            is_renaming: false,
            rename_buffer: String::new(),
        }
    }
}

struct MTabViewer<'a> {
    added_nodes: &'a mut Vec<(SurfaceIndex, NodeIndex)>,
    current_file: &'a mut String,
    content: &'a mut Content,
    vault: &'a str,
    icon_manager: &'a mut IconManager,
}

impl TabViewer for MTabViewer<'_> {
    type Tab = Tabe;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        let mut title = tab.title.clone();
        if self.icon_manager.settings.icon_in_title_enabled {
            let relative_path = if tab.path.starts_with(self.vault) {
                let p = tab.path.strip_prefix(self.vault).unwrap_or(&tab.path);
                p.strip_prefix('/').unwrap_or(p)
            } else {
                &tab.path
            };
            
            if let Some(icon_id) = self.icon_manager.get_icon(relative_path).filter(|s| !s.is_empty()) {
                      // Only prepend if it is NOT a byte-source (i.e. it is emoji or text)
                      // This check might be slow if get_icon_source reads disk. 
                      // Ideally we should check if it looks like an emoji or check svg_cache presence directly.
                      if !self.icon_manager.svg_cache.contains_key(icon_id) && !self.icon_manager.legacy_mappings.contains_key(icon_id) {
                          title = format!("{} {}", icon_id, title);
                      }
            }
        }
        title.into()
    }

    fn icon(&mut self, tab: &mut Self::Tab) -> Option<egui::Image<'static>> {
        if !self.icon_manager.settings.icon_in_title_enabled {
            return None;
        }

        let relative_path = if tab.path.starts_with(self.vault) {
            let p = tab.path.strip_prefix(self.vault).unwrap_or(&tab.path);
            p.strip_prefix('/').unwrap_or(p)
        } else {
            &tab.path
        };

        let icon_id = self.icon_manager.get_icon(relative_path).filter(|s| !s.is_empty())?;

        if let Some(IconSource::Bytes(bytes)) = self.icon_manager.get_icon_source(icon_id) {
             Some(egui::Image::from_bytes(
                 format!("bytes://title_{}.svg", icon_id),
                 bytes,
             ))
        } else {
             None
        }
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 5.0;
            if ui.add_enabled(tab.history_index > 0, egui::Button::new("⬅")).clicked() {
                tab.history_index -= 1;
                *self.current_file = tab.history[tab.history_index].clone();
                update_tab_content(tab, self.current_file, true);
            }
            if ui.add_enabled(tab.history_index + 1 < tab.history.len(), egui::Button::new("➡")).clicked() {
                tab.history_index += 1;
                *self.current_file = tab.history[tab.history_index].clone();
                update_tab_content(tab, self.current_file, true);
            }

            if tab.is_renaming {
                let res = ui.add(egui::TextEdit::singleline(&mut tab.rename_buffer).frame(false));
                if res.lost_focus() || (res.changed() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                    tab.is_renaming = false;
                    let old_path = Path::new(&tab.path);
                    if let Some(parent) = old_path.parent() {
                        let new_path = parent.join(&tab.rename_buffer);
                        if std::fs::rename(&tab.path, &new_path).is_ok() {
                            let old_rel = tab.path.strip_prefix(self.vault).unwrap_or(&tab.path);
                            let old_rel = old_rel.strip_prefix('/').unwrap_or(old_rel);
                            
                            let new_path_str = new_path.to_str().unwrap().to_string();
                            
                            let new_rel = new_path_str.strip_prefix(self.vault).unwrap_or(&new_path_str);
                            let new_rel = new_rel.strip_prefix('/').unwrap_or(new_rel);

                            self.icon_manager.rename_icon(self.vault, old_rel, new_rel);

                            tab.path = new_path_str.clone();
                            tab.title = tab.rename_buffer.clone();
                            *self.current_file = new_path_str.clone();
                            // Update history entry
                            tab.history[tab.history_index] = new_path_str;
                        }
                    }
                }
            } else {
                let label = ui.label(&tab.title).interact(egui::Sense::click());
                if label.double_clicked() {
                    tab.is_renaming = true;
                    tab.rename_buffer = tab.title.clone();
                }
            }

            // Selector de vista para archivos Kanban
            let is_kanban_file = match &tab.content {
                TabContent::Kanban { .. } => true,
                TabContent::Markdown { editor, .. } => editor.code.contains("kanban-plugin: board"),
                _ => false,
            };

            if is_kanban_file {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let is_kanban_view = matches!(tab.content, TabContent::Kanban { .. });
                    
                    if ui.selectable_label(!is_kanban_view, "📝 Text").clicked() && is_kanban_view {
                        let mut editor = easy_mark::EasyMarkEditor::default();
                        editor.code = files::read_file(&tab.path);
                        tab.content = TabContent::Markdown {
                            editor,
                            cache: CommonMarkCache::default(),
                        };
                    }
                    if ui.selectable_label(is_kanban_view, "📋 Board").clicked() && !is_kanban_view {
                        let mut gui = kanban::KanbanGui::default();
                        gui.set_path(&tab.path);
                        tab.content = TabContent::Kanban {
                            path: tab.path.clone(),
                            gui,
                        };
                    }
                });
            }
        });
        ui.separator();

        match &mut tab.content {
            TabContent::Graph { vault_path, state } => {
                if state.is_none() {
                    *state = Some(Box::new(crate::graph::Graph::new(
                        vault_path,
                        ui.ctx(),
                    )));
                }

                if let Some(graph) = state {
                    //graph.ui(ui, self.current_file, self.content, vault_path);

                    crate::graph::draw_ui(
                        graph,
                        ui,
                        self.current_file,
                        self.content,
                        vault_path,
                    );
                }
            }
            TabContent::Excalidraw { path, gui } => {
                gui.set_path(path);
                gui.show(ui, self.vault);
            }
            TabContent::Image(image_path) => {
                egui::ScrollArea::vertical()
                    .id_salt(format!("{}", tab.id))
                    .show(ui, |ui| {
                        let img = Image::from_uri(format!("file://{}", image_path));
                        ui.add(img);
                    });
            }
            TabContent::Income { path, gui } => {
                gui.set_path(path);
                gui.show(ui);
            }
            TabContent::Tasks { path, gui } => {
                gui.set_path(path);
                gui.show(ui);
            }
            TabContent::Kanban { path, gui } => {
                gui.set_path(path);
                if let Some(new_path) = gui.show(ui, self.vault) {
                     *self.current_file = new_path.clone();
                     update_tab_content(tab, &new_path, false);
                }
            }
            TabContent::Markdown { editor, cache } => {
                if tab.ctype == Content::View {
                    let cont = StripBuilder::new(ui)
                        .size(Size::relative(0.15))
                        .size(Size::relative(0.65));
                    cont.horizontal(|mut strip| {
                        strip.cell(|_| {});
                        strip.cell(|ui| {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                editor.code = files::read_file(&tab.path);
                                let frame =
                                    Frame::NONE.inner_margin(egui::Margin::symmetric(30, 10));
                                let inner_response = frame.show(ui, |ui| {
                                    let (markdown_content, metadata) = files::contents(&editor.code);
                                    
                                    // Debug prints
                                    // println!("DEBUG: icon_in_title_enabled: {}", self.icon_manager.settings.icon_in_title_enabled);
                                    // println!("DEBUG: tab.path: {}", tab.path);
                                    // println!("DEBUG: self.vault: {}", self.vault);

                                    if self.icon_manager.settings.icon_in_title_enabled {
                                        let relative_path = if tab.path.starts_with(self.vault) {
                                            let p = tab.path.strip_prefix(self.vault).unwrap_or(&tab.path);
                                            p.strip_prefix('/').unwrap_or(p).to_string()
                                        } else {
                                            tab.path.clone()
                                        };
                                        
                                        let icon_str = self.icon_manager.get_icon(&relative_path);
                                        
                                        if let Some(icon_id) = icon_str.filter(|s| !s.is_empty()) {
                                            ui.horizontal(|ui| {
                                                 if let Some(IconSource::Bytes(bytes)) = self.icon_manager.get_icon_source(icon_id) {
                                                     ui.add(egui::Image::from_bytes(
                                                         format!("bytes://title_{}.svg", icon_id),
                                                         bytes,
                                                     ).fit_to_exact_size(egui::vec2(20.0, 20.0)));
                                                 } else {
                                                     // Fallback for emojis
                                                     ui.label(egui::RichText::new(icon_id).size(20.0));
                                                 }
                                                 ui.heading(&tab.title);
                                            });
                                        } else {
                                             ui.heading(&tab.title);
                                        }
                                    } else {
                                        ui.heading(&tab.title);
                                    }

                                    if !metadata.is_empty() {
                                        create_metadata(metadata, ui);
                                    }
                                    
                                    // Store context for the 'static closure
                                    let ctx = ui.ctx().clone();
                                    ctx.data_mut(|d| {
                                        d.insert_temp(egui::Id::new("nav_vault"), self.vault.to_string());
                                        d.insert_temp(egui::Id::new("nav_current_path"), tab.path.clone());
                                    });

                                    CommonMarkViewer::new()
                                        .process_link(Some(&|ui, url, layout| {
                                            let response = ui.link(layout);
                                            if response.clicked() {
                                                let ctx = ui.ctx();
                                                let vault: String = ctx.data(|d| d.get_temp(egui::Id::new("nav_vault")).unwrap_or_default());
                                                let current_path: String = ctx.data(|d| d.get_temp(egui::Id::new("nav_current_path")).unwrap_or_default());
                                                
                                                let decoded_url = percent_encoding::percent_decode_str(url).decode_utf8_lossy().to_string();
                                                let clean_url = decoded_url.split('|').next().unwrap_or(&decoded_url).trim();
                                                
                                                let resolved = crate::files::resolve_path(&vault, &current_path, clean_url);

                                                if let Some(path) = resolved {
                                                    ctx.data_mut(|d| d.insert_temp(egui::Id::new("global_nav_request"), Some(path)));
                                                }
                                            }
                                            true 
                                        }))
                                        .show(ui, cache, &markdown_content);
                                    
                                    ui.allocate_space(ui.available_size());
                                });

                                if ui.input(|i| i.pointer.button_double_clicked(egui::PointerButton::Primary)) {
                                    if ui.rect_contains_pointer(inner_response.response.rect) {
                                        tab.ctype = Content::Edit;
                                    }
                                }
                            });
                        });
                    });
                } else if tab.ctype == Content::Edit {
                    let cont = StripBuilder::new(ui)
                        .size(Size::relative(0.25))
                        .size(Size::relative(0.5));
                    cont.horizontal(|mut strip| {
                        strip.cell(|_| {});
                        strip.cell(|ui| {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                let response = editor.ui(ui);
                                if response.changed() {
                                    let mut f = std::fs::OpenOptions::new()
                                        .write(true)
                                        .truncate(true)
                                        .open(&tab.path)
                                        .unwrap();
                                    f.write_all(editor.code.as_bytes()).unwrap();
                                    f.flush().unwrap();
                                }
                                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                                    tab.ctype = Content::View;
                                }
                            });
                        });
                    });
                }
            }
            TabContent::Empty => {
                ui.label("Empty tab content.");
            }
        }

        let nav_req: Option<String> = ui.ctx().data_mut(|d| d.get_temp(egui::Id::new("global_nav_request")).flatten());
        if let Some(path) = nav_req {
             ui.ctx().data_mut(|d| d.insert_temp(egui::Id::new("global_nav_request"), None::<String>));
             *self.current_file = path.clone();
             update_tab_content(tab, &path, false);
        }
    }
    fn on_add(&mut self, surface: crate::egui_dock::SurfaceIndex, node: NodeIndex) {
        self.added_nodes.push((surface, node))
    }
}
// Free function to update tab content
fn update_tab_content(tab: &mut Tabe, path: &String, is_history_nav: bool) {
    if !is_history_nav {
        if &tab.path == path {
            return;
        }
        // New navigation: truncate history after current index and push new path
        tab.history.truncate(tab.history_index + 1);
        tab.history.push(path.clone());
        tab.history_index = tab.history.len() - 1;
    }

    let new_title = Path::new(path)
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new("untitled"))
        .to_str()
        .unwrap_or("untitled")
        .to_string();

    tab.path = path.clone();
    tab.title = new_title;
    let loaded_content = files::read_file(path);
    if loaded_content.trim().is_empty() {
        tab.ctype = Content::Edit;
    } else {
        tab.ctype = Content::View;
    }

    tab.content = if path.ends_with(".png") || path.ends_with("jpeg") || path.ends_with("jpg") {
        TabContent::Image(path.clone())
    } else if path.ends_with(".inc") {
        let mut gui = income::IncomeGui::default();
        gui.set_path(path);
        TabContent::Income {
            path: path.clone(),
            gui,
        }
    } else if path.ends_with(".excalidraw.md") {
        let mut gui = excalidraw::ExcalidrawGui::default();
        gui.set_path(path);
        TabContent::Excalidraw {
            path: path.clone(),
            gui,
        }
    } else if path.ends_with(".graph") {
        let mut gui = tasks::TasksGui::default();
        gui.set_path(path);
        TabContent::Tasks {
            path: path.clone(),
            gui,
        }
    } else if loaded_content.contains("kanban-plugin: board") {
        let mut gui = kanban::KanbanGui::default();
        gui.set_path(path);
        TabContent::Kanban {
            path: path.clone(),
            gui,
        }
    } else {
        let mut editor = easy_mark::EasyMarkEditor::default();
        editor.code = loaded_content;
        TabContent::Markdown {
            editor,
            cache: CommonMarkCache::default(),
        }
    };
}

pub struct Tabs {
    //dock_state: DockState<Tabe>,
    tree: DockState<Tabe>,
    counter: usize,
}

impl Tabs {
    pub fn new_from_dock_state(dock_state: DockState<Tabe>) -> Self {
        let counter = dock_state
            .iter_all_tabs()
            .map(|(_, tab)| tab.id)
            .max()
            .unwrap_or(0)
            + 1;
        Self {
            tree: dock_state,
            counter,
        }
    }

    pub fn new_empty() -> Self {
        Self {
            tree: DockState::new(vec![]),
            counter: 0,
        }
    }

    pub fn ui(
        &mut self,
        ui: &mut Ui,
        _marker: &mut crate::graph::Graph,
        current_file: &mut String,
        content: &mut Content,
        vault: &str,
        icon_manager: &mut IconManager,
        dock_style: &Style,
    ) {
        if ui.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::W)) {
            if let Some((focus_surf, focus_node)) = self.tree.focused_leaf() {
                let mut to_remove = None;

                for (s_idx, surface) in self.tree.iter_surfaces().enumerate() {
                    if crate::egui_dock::SurfaceIndex(s_idx) == focus_surf {
                        for (n_idx, node) in surface.iter_nodes().enumerate() {
                            if crate::egui_dock::NodeIndex(n_idx) == focus_node {
                                if let crate::egui_dock::Node::Leaf(leaf) = node {
                                    to_remove = Some((focus_surf, focus_node, leaf.active));
                                }
                                break;
                            }
                        }
                    }
                    if to_remove.is_some() {
                        break;
                    }
                }

                if let Some(target) = to_remove {
                    self.tree.remove_tab(target);
                }
            }
        }

        let mut added_nodes = Vec::new();
        let tab_viewer = &mut MTabViewer {
            added_nodes: &mut added_nodes,
            //graph: marker,
            current_file,
            content,
            vault,
            icon_manager,
        };
        DockArea::new(&mut self.tree)
            .style(dock_style.clone())
            .show_add_buttons(true)
            .show_inside(ui, tab_viewer);
        added_nodes.drain(..).for_each(|(surface, node)| {
            self.tree.set_focused_node_and_surface((surface, node));
            let last = self.tree.iter_all_tabs().last();
            let cloned_path = last.unwrap().1.path.clone();
            self.counter += 1;
            self.tree
                .push_to_focused_leaf(Tabe::new(self.counter, cloned_path));
        });
    }

    pub fn file_changed(&mut self, path: &str) {
        if self.tree.iter_all_tabs().count() == 0 {
            self.counter += 1;
            self.tree = DockState::new(vec![Tabe::new(self.counter, path.to_string())]);
            return;
        }

        if let Some((_, tab)) = self.tree.find_active_focused() {
            update_tab_content(tab, &path.to_string(), false);
        } else {
            self.counter += 1;
            let new_tab = Tabe::new(self.counter, path.to_string());
            self.tree.push_to_first_leaf(new_tab);
        }
    }

    pub fn add_tab(&mut self, tab: Tabe) {
        self.tree.push_to_focused_leaf(tab);
    }

    pub fn dock_state(&self) -> &DockState<Tabe> {
        &self.tree
    }
}
