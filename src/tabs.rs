use crate::excalidraw;
use crate::easy_mark;
use crate::files;
use crate::format;
use crate::income;

use crate::main_area::content_enum::Content;
use crate::main_area::metadata_renderer::create_metadata;
use crate::tasks;
use egui::Image;
use egui::{FontId, Frame, Sense, Ui, WidgetText};
use egui_commonmark::*;
use egui_dock::{DockArea, DockState, NodeIndex, Style, SurfaceIndex, TabViewer};
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

        let (initial_ctype, initial_buffer) = if loaded_content.trim().is_empty() {
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
        } else if path.ends_with(".excalidraw") {
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
            path,
            content,
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
        }
    }
}

struct MTabViewer<'a> {
    added_nodes: &'a mut Vec<(SurfaceIndex, NodeIndex)>,
    current_file: &'a mut String,
    content: &'a mut Content,
    vault: &'a str,
}

impl TabViewer for MTabViewer<'_> {
    type Tab = Tabe;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        tab.title.as_str().into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
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
                gui.show(ui);
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
                                    ui.heading(&tab.title);
                                    if !metadata.is_empty() {
                                        create_metadata(metadata, ui);
                                    }
                                    // Use CommonMarkViewer for viewing as it was before, 
                                    // or easy_mark if preferred. The user asked for easy_mark editor.
                                    CommonMarkViewer::new().show(ui, cache, &markdown_content);
                                    ui.allocate_space(ui.available_size());
                                });

                                let interact_response = ui.interact(
                                    inner_response.response.rect,
                                    ui.id().with("frame_interact"),
                                    Sense::click(),
                                );

                                if interact_response.double_clicked() {
                                    tab.ctype = Content::Edit;
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
    }
    fn on_add(&mut self, surface: egui_dock::SurfaceIndex, node: NodeIndex) {
        self.added_nodes.push((surface, node))
    }
}
// Free function to update tab content
fn update_tab_content(tab: &mut Tabe, path: &String) {
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
    } else if path.ends_with(".excalidraw") {
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
    ) {
        if ui.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::W)) {
            if let Some((focus_surf, focus_node)) = self.tree.focused_leaf() {
                let mut to_remove = None;

                for (s_idx, surface) in self.tree.iter_surfaces().enumerate() {
                    if egui_dock::SurfaceIndex(s_idx) == focus_surf {
                        for (n_idx, node) in surface.iter_nodes().enumerate() {
                            if egui_dock::NodeIndex(n_idx) == focus_node {
                                if let egui_dock::Node::Leaf(leaf) = node {
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
        };
        DockArea::new(&mut self.tree)
            .style(Style::from_egui(ui.style().as_ref()))
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

    pub fn file_changed(&mut self, path: String) {
        if self.tree.iter_all_tabs().count() == 0 {
            self.counter += 1;
            self.tree = DockState::new(vec![Tabe::new(self.counter, path)]);
            return;
        }

        if let Some((_, tab)) = self.tree.find_active_focused() {
            update_tab_content(tab, &path);
        } else {
            self.counter += 1;
            let new_tab = Tabe::new(self.counter, path);
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
