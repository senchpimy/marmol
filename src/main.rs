use egui::*;
use std::fmt;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[macro_use]
extern crate json;

mod configuraciones;
mod files;
mod format;
mod graph;
mod income;
mod main_area;
mod screens;
mod search;
mod server;
mod tabs;
mod tasks;
mod toggle_switch;

#[derive(PartialEq, Debug)]
enum NewFileType {
    Markdown,
    Income,
    Tasks,
}
struct MShape {
    height: f32,
    width: f32,
}

impl fmt::Display for NewFileType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        ..Default::default()
    };
    eframe::run_native(
        "Marmol",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Box::new(Marmol::new(cc))
        }),
    )
}

struct Marmol {
    prev_current_file: String,
    new_vault_str: String,
    content: main_area::Content,

    current_window: screens::Screen,
    prev_window: screens::Screen,
    config_path: String,
    left_controls: main_area::LeftControls,
    new_file_str: String,

    left_collpased: bool,
    vault: String,
    vault_vec: Vec<String>,
    current_file: String,
    window_size: MShape,

    create_new_vault: bool,
    create_file_error: String,
    show_create_button: bool,
    new_vault_folder: String,
    new_vault_folder_err: String,
    vault_changed: bool,
    font_size: f32,
    center_size: f32,
    center_size_remain: f32,
    sort_files: bool,
    tabs: tabs::Tabs,

    new_file_type: NewFileType,
    marker: graph::Graph,
}

impl Marmol {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let font_size = configuraciones::load_context();
        let ctx = &cc.egui_ctx;
        ctx.style_mut(|style| {
            let font_id = FontId::proportional(font_size);
            style.override_font_id = Some(font_id);
        });
        //ctx.set_visuals(configuraciones::load_colors());
        Self {
            font_size,
            ..Default::default()
        }
    }
}

impl Default for Marmol {
    fn default() -> Self {
        let (
            vault_var,     //graph_json_config
            vault_vec_var, //Vec de diferentes vaults
            current,
            config_path_var,
            window,
            left_coll,
            center_size,
            sort_files,
        ) = configuraciones::load_vault();
        println!("{}", current);
        Self {
            window_size: MShape {
                height: 0.,
                width: 0.,
            },
            tabs: tabs::Tabs::new(current.clone()),
            center_size,
            center_size_remain: (1.0 - center_size) / 2.0,
            font_size: 12.0,
            marker: graph::Graph::new(&vault_var),
            new_file_str: String::new(),
            content: main_area::Content::View,
            left_controls: main_area::LeftControls::default(),
            new_vault_folder: String::from(""),
            new_vault_folder_err: String::from(""),
            new_vault_str: String::from(""),
            config_path: config_path_var.to_owned(),
            create_new_vault: false,
            show_create_button: false,
            current_window: window,
            prev_window: window,
            prev_current_file: current.to_owned(),
            create_file_error: String::new(),
            vault: vault_var,
            vault_vec: vault_vec_var,
            current_file: current.to_owned(),
            new_file_type: NewFileType::Markdown,

            left_collpased: left_coll,
            vault_changed: false,
            sort_files, //right_collpased:true,
        }
    }
}

impl eframe::App for Marmol {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.input(|i| match i.viewport().outer_rect {
            Some(a) => {
                //a.min; // Position
                self.window_size = MShape {
                    width: a.max.x,
                    height: a.max.y,
                };
            }
            None => {}
        });
        if self.current_window == screens::Screen::Default {
            //welcome screen
            screens::default(
                ctx,
                &mut self.current_window,
                &mut self.new_vault_str,
                &mut self.vault_vec,
                &mut self.vault,
                &mut self.content,
            );
        } else if self.current_window == screens::Screen::Main {
            //Main screen
            self.left_controls.left_side_settings(
                ctx,
                &mut self.left_collpased,
                &mut self.vault,
                &mut self.current_file,
                &mut self.current_window,
                &mut self.content,
                &self.window_size,
            );
            self.left_controls.left_side_menu(
                ctx,
                &self.left_collpased,
                &self.vault,
                &mut self.current_file,
                &self.sort_files,
            );
            CentralPanel::default().show(ctx, |ui| {
                if self.prev_current_file != self.current_file {
                    self.prev_current_file = self.current_file.clone(); //TODO remove
                    {
                        self.tabs.file_changed(self.current_file.clone());
                    }
                }
                self.tabs.ui(ui);
                if self.content == main_area::Content::NewFile {
                    self.new_file(ui, ctx.input(|i| i.key_pressed(Key::Enter)));
                }
                /*self.buffer = files::read_file(&self.current_file);
                                        ui.label("✏");
                                            RichText::new("👁")
                    //}else if self.content == main_area::Content::NewTask{
                    //    self.new_file(ui,ctx.input(|i| i.key_pressed(Key::Enter)));
                    } else if self.content == main_area::Content::Graph {
                        self.marker.ui(
                            ui,
                            &mut self.current_file,
                            &mut self.content,
                            &self.vault,
                        );
                        self.marker.controls(ctx);
                    }
                }); //termina CentralPanel
                    //Termina Principal*/
            });
        } else if self.current_window == screens::Screen::Configuracion {
            //TODO fix this mess
            screens::configuracion(
                ctx,
                &mut self.prev_window,
                &mut self.current_window,
                &mut self.vault_vec,
                &mut self.vault,
                &mut self.new_vault_str,
                &mut self.create_new_vault,
                &mut self.new_vault_folder,
                &mut self.new_vault_folder_err,
                &mut self.show_create_button,
                &mut self.vault_changed,
                &mut self.font_size,
                &mut self.center_size,
                &mut self.center_size_remain,
                &mut self.sort_files,
            );
            if self.vault_changed {
                self.marker.update_vault(Path::new(&self.vault));
            }
        } else if self.current_window == screens::Screen::Server {
            screens::set_server(ctx);
        };
        /////////////////////////////////////////////////////////////////////////////////
    }

    //TODO replace with serde?
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        let vault_str = format!("vault: '{}'", &self.vault);
        //let mut vec_str = String::new();
        //for i in &self.vault_vec {
        //    vec_str = vec_str.to_owned() + format!(" '{}' ,", &i).as_str();
        //}
        let vec_str: String = self
            .vault_vec
            .iter()
            .map(|item| format!("'{}'", item))
            .collect::<Vec<String>>()
            .join(", ");

        let dir = Path::new(&self.config_path);
        println!("{}", &self.config_path);
        if !dir.exists() {
            _ = fs::create_dir(&self.config_path);
        }
        let vault_vec_str = format!("vault_vec: [ {} ]", vec_str);
        let file_path = String::from(&self.config_path) + "/ProgramState";
        let current_file = format!("current: {}", &self.current_file);
        let center_size = format!("center_size: {}", &self.center_size);
        let left_menu = format!("left_menu: {}", &self.left_collpased);
        let sort_files = format!("sort_files: {}", &self.sort_files);
        let new_content = format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            &vault_vec_str, vault_str, current_file, left_menu, center_size, sort_files
        );
        let mut file = fs::File::create(file_path).unwrap();
        file.write_all(new_content.as_bytes()).unwrap();

        let context_path = String::from(&self.config_path) + "/ContextState";
        let mut file2 = fs::File::create(context_path).unwrap();
        let font_size = format!("font_size: {}", &self.font_size);
        //let context_contents=font_size;
        file2.write_all(font_size.as_str().as_bytes()).unwrap();
    }
}

impl Marmol {
    fn new_file(&mut self, ui: &mut Ui, enter_clicked: bool) {
        ui.label("Create New File");
        ui.add(egui::TextEdit::singleline(&mut self.new_file_str));
        let new_path = format!("{}/{}", &self.vault, &self.new_file_str);
        egui::ComboBox::from_label("Editar categoria")
            .selected_text(&self.new_file_type.to_string())
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut self.new_file_type, NewFileType::Markdown, "Markdown");
                ui.selectable_value(&mut self.new_file_type, NewFileType::Tasks, "Tasks");
                ui.selectable_value(&mut self.new_file_type, NewFileType::Income, "Income");
            });
        let path = if self.new_file_type == NewFileType::Tasks {
            format!("{}.graph", new_path)
        } else if self.new_file_type == NewFileType::Income {
            format!("{}.inc", new_path)
        } else {
            String::new()
        };
        let new_file = if self.new_file_type == NewFileType::Markdown {
            Path::new(&new_path)
        } else {
            Path::new(&path)
        };
        ui.label(RichText::new(&self.create_file_error).color(Color32::RED));
        if new_file.exists() {
            self.create_file_error = String::from("File already exist");
        } else {
            if ui.button("Create").clicked() || enter_clicked {
                self.content = main_area::Content::View;
                let res = File::create(new_file);
                match res {
                    Ok(mut re) => {
                        self.create_file_error = String::new();
                        if self.new_file_type == NewFileType::Tasks {
                            let contents = String::from("{\"tasks\":[],\"days\":[],\"top_id\":0}");
                            re.write_all(contents.as_bytes()).unwrap();
                        } else if self.new_file_type == NewFileType::Income {
                            let contents=String::from("{\"transacciones\":[],\"categorias\":[ \"Categoria\"],\"colores\":[[0.0,0.0,0.0]]}");
                            re.write_all(contents.as_bytes()).unwrap();
                        }
                        self.current_file = String::from(new_file.to_str().unwrap());
                    }
                    Err(x) => {
                        self.create_file_error = x.to_string();
                    }
                }
                self.new_file_str = String::new();
            }
            self.create_file_error = String::new();
        }
        if ui.button("Cancel").clicked() {
            self.content = main_area::Content::View;
            self.new_file_str = String::new();
        }
    }
}
