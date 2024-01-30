use crate::files;
use crate::income;
use crate::main_area;
use crate::tasks;
use egui::epaint::image;
use egui::Image;
use egui::{FontId, Ui, WidgetText};
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
    //image:Option<Image<'static>>
    buffer_image: Vec<u8>,
}

impl Tabe {
    fn new(n: usize, path: String) -> Self {
        let title = Path::new(&path)
            .file_name()
            .unwrap()
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
            //image:None
            buffer_image: Vec::new(),
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
        if (tab.path.ends_with(".png") || tab.path.ends_with("jpeg") || tab.path.ends_with("jpg")) && true {
            tab.buffer_image = Vec::new();
            tab.buffer_image = files::read_image(&tab.path);
            //tab.image = Some(Image::from_uri(tab.path.clone()));
            tab.is_image = true;
            //println!("{}",tab.path);
        } else {
            tab.is_image = false;
        }

        if tab.is_image {
           // match tab.image.clone() {
           //     Some(x)=>{
           //         ui.add(x.clone());//TODO ?????
           //     }
           //     None=>{}
           // }
            egui::ScrollArea::vertical()
                .id_source(format!("{}", tab.id))
                .show(ui, |ui| {
                    add_image(ui, &tab.buffer_image, tab.id);
                });
        }else if tab.ctype == main_area::Content::View {
            let cont = StripBuilder::new(ui)
                .size(Size::relative(0.3))
                .size(Size::relative(0.3));
            cont.horizontal(|mut strip| {
                strip.cell(|_| {});
                strip.cell(|ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        tab.content = files::read_file(&tab.path); //TODO Read Just once
                        let (content, metadata) = files::contents(&tab.content);
                        ui.heading(&tab.title);
                        if !metadata.is_empty() {
                            main_area::create_metadata(metadata, ui);
                        }
                        CommonMarkViewer::new(tab.id).show(ui, &mut tab.common_mark_c, &content);
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
                        //Why ?
                        /*if ctx.input(|i| i.key_pressed(Key::Enter)) && response.has_focus() {
                            let mut f = std::fs::OpenOptions::new()
                                .write(true)
                                .truncate(true)
                                .open(&tab.path)
                                .unwrap();
                            f.write_all(format::indent(&self.text_edit).as_bytes())
                                .unwrap();
                            f.flush().unwrap();
                        }*/
                    });
                });
            });
        } else if tab.path.ends_with(".inc") {
            tab.income.set_path(&tab.path);
            tab.income.show(ui);
        } else if tab.path.ends_with(".graph") {
            tab.tasks.set_path(&tab.path);
            tab.tasks.show(ui);
        }
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

    pub fn file_changed(&mut self,path:String){
        match self.tree.find_active_focused() {
            None=>{},
            Some((_,obj))=>{
                obj.path= path;
                //self.tree.push_to_focused_leaf(Tabe::new(1, path));
            }
        }
    }
}

//fn add_image<'a>(ui: &'a mut egui::Ui, vec: &'a Vec<u8>) -> Image<'a> {
fn add_image(ui: &mut egui::Ui, vec: &Vec<u8>, id: usize) {
    let mut img = Image::from_bytes(format!("{id}"), vec.clone());
    let image_size = img.size().unwrap_or(egui::Vec2::default()); // If its loaded
                                                                  // with bytes it will return none
                                                                  //if image_size[0] > self.window_size.width {
                                                                  //    img = img.max_width(self.window_size.width);
    ui.add(img);
    //};
    //img
}
