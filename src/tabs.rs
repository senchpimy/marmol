use crate::files;
use crate::main_area;
use egui::{Ui, WidgetText};
use egui_commonmark::*;
use egui_dock::{DockArea, DockState, NodeIndex, Style, SurfaceIndex, TabViewer};
use egui_extras::{Size, StripBuilder};
use std::path::Path;

//type Tabe = String;
struct Tabe {
    title: String,
    path: String,
    buffer: String,
}

impl Tabe {
    fn new(n: usize, path: String) -> Self {
        Self {
            title: format!("TAB NUM {n}"),
            path,
            buffer: String::new(),
        }
    }
}

struct MyTabViewer<'a> {
    added_nodes: &'a mut Vec<(SurfaceIndex, NodeIndex)>,
}

impl TabViewer for MyTabViewer<'_> {
    type Tab = Tabe;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        tab.title.as_str().into()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        //ui.label(format!("Content of {}", tab.title));
        //if ctx.input(|i| i.key_pressed(Key::F)) {
        //    println!("Search");
        //}
        let cont = StripBuilder::new(ui)
            .size(Size::relative(0.3))
            .size(Size::relative(0.3));
        cont.horizontal(|mut strip| {
            strip.cell(|_| {});
            strip.cell(|ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let header = Path::new(&tab.path).file_name().unwrap();
                    let tmp_buffer = files::read_file(&tab.path);
                    let (content, metadata) = files::contents(&tmp_buffer);
                    ui.heading(header.to_str().unwrap());
                    if !metadata.is_empty() {
                        main_area::create_metadata(metadata, ui);
                    }
                    CommonMarkViewer::new("v").show(ui, &mut CommonMarkCache::default(), &content);
                });
            });
        });
    }
    fn on_add(&mut self, surface: egui_dock::SurfaceIndex, node: NodeIndex) {
        self.added_nodes.push((surface, node))
    }
}
pub struct Tabs {
    //dock_state: DockState<Tabe>,
    tree: DockState<Tabe>,
    counter: usize,
}

impl Tabs {
    pub fn new(path: String) -> Self {
        let mut tabs = Vec::new();
        tabs.push(Tabe::new(0, path));
        let tree = DockState::new(tabs);
        Self { tree, counter: 0 }
    }
}

impl Tabs {
    pub fn ui(&mut self, ui: &mut Ui) {
        let mut added_nodes = Vec::new();
        DockArea::new(&mut self.tree)
            .style(Style::from_egui(ui.style().as_ref()))
            .show_add_buttons(true)
            .show_inside(
                ui,
                &mut MyTabViewer {
                    added_nodes: &mut added_nodes,
                },
            );
        added_nodes.drain(..).for_each(|(surface, node)| {
            self.tree.set_focused_node_and_surface((surface, node));
            let last = self.tree.iter_all_tabs().last();
            let cloned_path = last.unwrap().1.path.clone();
            self.tree
                .push_to_focused_leaf(Tabe::new(self.counter, cloned_path));
            self.counter += 1;
        });
    }
}
