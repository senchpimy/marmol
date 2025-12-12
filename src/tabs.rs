use crate::files;
use crate::income;
use crate::main_area;
use crate::tasks;
use egui::Image;
use egui::{FontId, Frame, Sense, Ui, WidgetText};
use egui_commonmark::*;
use egui_dock::{DockArea, DockState, NodeIndex, Style, SurfaceIndex, TabViewer};
use egui_extras::{Size, StripBuilder};
use std::io::Write;
use std::path::Path;

//type Tabe = String;
struct Tabe {
    id: usize,
    title: String,
    path: String,
    content: String,
    buffer: String,
    is_image: bool,
    ctype: main_area::Content,
    common_mark_c: CommonMarkCache,
    income: income::IncomeGui,
    tasks: tasks::TasksGui,
}

impl Tabe {
    fn new(n: usize, path: String) -> Self {
        dbg!(&path);
        let title = Path::new(&path)
            .file_name()
            .unwrap()
            //.unwrap_or("")
            .to_str()
            .unwrap()
            .into();
        Self {
            id: n,
            ctype: main_area::Content::View,
            title,
            path,
            content: String::new(),
            buffer: String::new(),
            is_image: false,
            common_mark_c: CommonMarkCache::default(),
            income: income::IncomeGui::default(),
            tasks: tasks::TasksGui::default(),
        }
    }
}

struct MTabViewer<'a> {
    added_nodes: &'a mut Vec<(SurfaceIndex, NodeIndex)>,
}

impl TabViewer for MTabViewer<'_> {
    type Tab = Tabe;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        tab.title.as_str().into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        //ui.label(format!("Content of {}", tab.title));
        //if ctx.input(|i| i.key_pressed(Key::F)) {
        //    println!("Search");
        //}
        if tab.path.ends_with(".png") || tab.path.ends_with("jpeg") || tab.path.ends_with("jpg") {
            tab.is_image = true;
        } else {
            tab.is_image = false;
        }

        if tab.is_image {
            egui::ScrollArea::vertical()
                .id_salt(format!("{}", tab.id))
                .show(ui, |ui| {
                    let img = Image::from_uri(format!("file://{}", &tab.path));
                    ui.add(img);
                });
        } else if tab.path.ends_with(".inc") {
            tab.income.set_path(&tab.path);
            tab.income.show(ui);
        } else if tab.path.ends_with(".graph") {
            tab.tasks.set_path(&tab.path);
            tab.tasks.show(ui);
        } else if tab.ctype == main_area::Content::View {
            //centrar los contenidos

            let cont = StripBuilder::new(ui)
                .size(Size::relative(0.3))
                .size(Size::relative(0.3));
            cont.horizontal(|mut strip| {
                strip.cell(|_| {});
                strip.cell(|ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        tab.content = files::read_file(&tab.path);
                        let frame = Frame::new();
                        let inner_response = frame.show(ui, |ui| {
                            let (content, metadata) = files::contents(&tab.content);
                            ui.heading(&tab.title);
                            if !metadata.is_empty() {
                                main_area::create_metadata(metadata, ui);
                            }
                            CommonMarkViewer::new().show(ui, &mut tab.common_mark_c, &content);
                        });

                        let interact_response = ui.interact(
                            inner_response.response.rect,
                            ui.id().with("frame_interact"),
                            Sense::click(),
                        );

                        if interact_response.double_clicked() {
                            tab.ctype = main_area::Content::Edit;
                            tab.buffer = tab.content.clone();
                        }
                    });
                });
            });
        } else if tab.ctype == main_area::Content::Edit {
            let cont = StripBuilder::new(ui)
                .size(Size::relative(0.3))
                .size(Size::relative(0.3));
            cont.horizontal(|mut strip| {
                strip.cell(|_| {});
                strip.cell(|ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let zone = egui::TextEdit::multiline(&mut tab.buffer)
                            .font(FontId::proportional(15.0));
                        let response = ui.add_sized(ui.available_size(), zone);
                        if response.changed() {
                            let mut f = std::fs::OpenOptions::new()
                                .write(true)
                                .truncate(true)
                                .open(&tab.path)
                                .unwrap();
                            f.write_all(tab.buffer.as_bytes()).unwrap();
                            f.flush().unwrap();
                        }
                        if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                            tab.ctype = main_area::Content::View;
                        }
                    });
                });
            });
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
    tab.ctype = main_area::Content::View;
    tab.content = String::new();
    tab.buffer = String::new();
    tab.common_mark_c = CommonMarkCache::default();
}

pub struct Tabs {
    //dock_state: DockState<Tabe>,
    tree: DockState<Tabe>,
    counter: usize,
}

impl Tabs {
    pub fn new(path: Option<String>) -> Self {
        dbg!(&path);
        let mut tabs = Vec::new();
        if let Some(path) = path {
            tabs.push(Tabe::new(0, path));
        }
        let tree = DockState::new(tabs);
        Self { tree, counter: 0 }
    }

    pub fn ui(&mut self, ui: &mut Ui) {
        let mut added_nodes = Vec::new();
        DockArea::new(&mut self.tree)
            .style(Style::from_egui(ui.style().as_ref()))
            .show_add_buttons(true)
            .show_inside(
                ui,
                &mut MTabViewer {
                    added_nodes: &mut added_nodes,
                },
            );
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
        dbg!("file changed");

        if self.tree.iter_all_tabs().count() == 0 {
            self.counter += 1;
            self.tree = DockState::new(vec![Tabe::new(self.counter, path)]);
            return;
        }

        if let Some((_, tab)) = self.tree.find_active_focused() {
            update_tab_content(tab, &path);
        } else {
            // If no tab is focused, create a new tab to display the content.
            self.counter += 1;
            let new_tab = Tabe::new(self.counter, path);
            self.tree.push_to_first_leaf(new_tab);
        }
    }
}
